import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// C# 服务器可执行文件路径
const SERVER_PATH = path.resolve(
  __dirname,
  '../../windows-bash-tools-mcp-csharp/BashToolsServer/bin/Debug/net10.0/bash-tools.exe',
);

const results: { name: string; pass: boolean; detail: string }[] = [];

function report(name: string, pass: boolean, detail: string): void {
  results.push({ name, pass, detail });
  console.log(`\n${pass ? '✅' : '❌'} [${name}] ${pass ? 'PASS' : 'FAIL'}`);
  console.log(`   ${detail}`);
}

function parseResult(result: any): any {
  const content = result.content as { type: string; text: string }[];
  if (!content || content.length === 0) {
    throw new Error('工具返回内容为空');
  }
  return JSON.parse(content[0].text);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function countPingReplies(output: string): number {
  const matches = output.match(/来自|Reply from/gi);
  return matches ? matches.length : 0;
}

function checkProcess(name: string): Promise<string[]> {
  return new Promise((resolve) => {
    const p = spawn('tasklist', []);
    let out = '';
    p.stdout.on('data', (d) => (out += d.toString()));
    p.on('close', () => {
      resolve(out.split('\n').filter((l) => l.toLowerCase().includes(name.toLowerCase())));
    });
  });
}

async function main(): Promise<void> {
  console.log('='.repeat(60));
  console.log('  C# Bash Tools 综合测试 (bash / bash_output / kill_shell)');
  console.log('='.repeat(60));
  console.log(`服务器路径: ${SERVER_PATH}\n`);

  const client = new Client({ name: 'test-csharp', version: '1.0.0' }, { capabilities: {} });
  const transport = new StdioClientTransport({ command: SERVER_PATH, args: [] });

  try {
    console.log('🚀 连接 MCP 服务器...');
    await client.connect(transport);
    console.log('✅ 连接成功\n');

    // ========== 步骤 0: 工具列表 ==========
    console.log('━'.repeat(60));
    console.log('📋 步骤 0: 列出可用工具 (tools/list)');
    const toolsResp = await client.listTools();
    const toolNames = toolsResp.tools.map((t: any) => t.name);
    console.log(`   暴露工具数: ${toolNames.length}`);
    toolNames.forEach((n: string) => console.log(`   - ${n}`));
    const expected = ['bash', 'bash_output', 'kill_shell'];
    const toolOk = expected.every((n) => toolNames.includes(n));
    report('工具暴露', toolOk, `期望 ${expected.join(', ')}, 实际 ${toolNames.join(', ')} (共 ${toolNames.length} 个)`);

    // ========== 步骤 1: bash 前台 (cmdlet) ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 1: bash 前台 cmdlet');
    const fg = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: 'Write-Output "HELLO-FROM-FG-$PID"', timeout: 10000, run_in_background: false },
      }),
    );
    console.log(`   exitCode=${fg.exitCode}`);
    console.log(`   Output: ${fg.output}`);
    const fgOk = fg.exitCode === 0 && String(fg.output).includes('HELLO-FROM-FG');
    report('bash 前台 cmdlet', fgOk, `exitCode=${fg.exitCode}, 含 HELLO-FROM-FG: ${String(fg.output).includes('HELLO-FROM-FG')}`);

    // ========== 步骤 2: bash 前台 中文编码 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 2: bash 前台 中文输出');
    const fgC = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: 'Write-Output "你好世界-中文测试"', timeout: 10000, run_in_background: false },
      }),
    );
    console.log(`   Output: ${fgC.output}`);
    const fgCOk = fgC.exitCode === 0 && String(fgC.output).includes('你好世界');
    report('中文编码', fgCOk, `exitCode=${fgC.exitCode}, 含中文: ${String(fgC.output).includes('你好世界')}`);

    // ========== 步骤 3: bash 前台 原生命令 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 3: bash 前台 原生命令 (node --version)');
    const fgN = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: '& node.exe --version', timeout: 10000, run_in_background: false },
      }),
    );
    console.log(`   exitCode=${fgN.exitCode}`);
    console.log(`   Output: ${fgN.output}`);
    const fgNOk = fgN.exitCode === 0 && String(fgN.output).includes('v');
    report('原生命令输出 (node)', fgNOk, `exitCode=${fgN.exitCode}, 输出=[${String(fgN.output).trim()}]`);

    // ========== 步骤 4: bash 前台 ping 原生命令 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 4: bash 前台 ping (原生)');
    const fgP = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: 'ping.exe 127.0.0.1 -n 2', timeout: 15000, run_in_background: false },
      }),
    );
    console.log(`   exitCode=${fgP.exitCode}`);
    console.log(`   Ping 回复数: ${countPingReplies(fgP.output ?? '')}`);
    const fgPOk = fgP.exitCode === 0 && countPingReplies(fgP.output ?? '') >= 2;
    report('原生命令输出 (ping)', fgPOk, `exitCode=${fgP.exitCode}, ping回复=${countPingReplies(fgP.output ?? '')}`);

    // ========== 步骤 5: bash 后台 + bash_output ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 5: bash 后台 (PowerShell 循环) + bash_output 增量');
    const bgCmd = 'for ($i=0; $i -lt 8; $i++) { Write-Output "TICK-$i $(Get-Date -Format HH:mm:ss)"; Start-Sleep -Milliseconds 500 }';
    const bg = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: bgCmd, timeout: 60000, run_in_background: true, description: '后台循环' },
      }),
    );
    const shellId = bg.shell_id;
    console.log(`   shell_id: ${shellId}`);
    const bgOk = typeof shellId === 'string' && shellId.length > 0;
    report('bash 后台', bgOk, `shell_id=${shellId ?? '(空)'}`);

    if (bgOk) {
      await sleep(1500);
      const o1 = parseResult(await client.callTool({ name: 'bash_output', arguments: { bash_id: shellId } }));
      console.log(`   第一次 status=${o1.status}, TICK数=${(o1.output ?? '').split('\n').filter((l: string) => l.includes('TICK')).length}`);
      await sleep(1500);
      const o2 = parseResult(await client.callTool({ name: 'bash_output', arguments: { bash_id: shellId } }));
      console.log(`   第二次 status=${o2.status}, TICK数=${(o2.output ?? '').split('\n').filter((l: string) => l.includes('TICK')).length}`);
      const c1 = (o1.output ?? '').split('\n').filter((l: string) => l.includes('TICK')).length;
      const c2 = (o2.output ?? '').split('\n').filter((l: string) => l.includes('TICK')).length;
      const outOk = o1.status === 'running' && c2 > c1 && c2 > 0;
      report('bash_output 增量', outOk, `第一次=${c1}, 第二次=${c2}, 状态=${o1.status}`);

      // ========== 步骤 6: bash_output 过滤 ==========
      console.log('\n' + '━'.repeat(60));
      console.log('📝 步骤 6: bash_output 正则过滤 (TICK)');
      const f1 = parseResult(
        await client.callTool({ name: 'bash_output', arguments: { bash_id: shellId, filter: 'TICK' } }),
      );
      const filteredLines = (f1.output ?? '').split('\n').filter((l: string) => l.trim().length > 0);
      const filterOk = filteredLines.length > 0 && filteredLines.every((l: string) => l.includes('TICK'));
      report('bash_output 过滤', filterOk, `过滤后 ${filteredLines.length} 行全部含 TICK: ${filterOk}`);

      // ========== 步骤 7: kill_shell ==========
      console.log('\n' + '━'.repeat(60));
      console.log('🛑 步骤 7: kill_shell 终止后台任务');
      const kill = parseResult(await client.callTool({ name: 'kill_shell', arguments: { shell_id: shellId } }));
      console.log(`   Message: ${kill.message}`);
      await sleep(1500);
      const afterKill = parseResult(await client.callTool({ name: 'bash_output', arguments: { bash_id: shellId } }));
      console.log(`   终止后查询: status=${afterKill.status} (任务已移除)`);
      const killOk = String(kill.message).includes('killed') && afterKill.status === 'not_found';
      report('kill_shell', killOk, `message=${kill.message}, 终止后=${afterKill.status}`);
    }

    // ========== 步骤 8: 前台超时自动转后台 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('⏱️ 步骤 8: 前台超时自动转后台');
    const to = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: 'Start-Sleep -Seconds 8; Write-Output "DONE"', timeout: 1000, run_in_background: false },
      }),
    );
    console.log(`   shell_id: ${to.shell_id}`);
    const toOk = typeof to.shell_id === 'string' && to.shell_id.length > 0;
    report('前台超时转后台', toOk, `shell_id=${to.shell_id ?? '(空)'}`);
    if (to.shell_id) {
      const tk = parseResult(await client.callTool({ name: 'kill_shell', arguments: { shell_id: to.shell_id } }));
      console.log(`   🧹 已清理超时任务: ${tk.message}`);
    }

    // ========== 步骤 9: 危险命令拦截 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('🛡️ 步骤 9: 危险命令拦截');
    const danger = parseResult(
      await client.callTool({
        name: 'bash',
        arguments: { command: 'Remove-Item -Recurse -Force C:\\temp', timeout: 10000, run_in_background: false },
      }),
    );
    console.log(`   exitCode=${danger.exitCode}`);
    console.log(`   Output: ${danger.output}`);
    const dangerOk = danger.exitCode === 1 && String(danger.output).includes('rejected');
    report('危险命令拦截', dangerOk, `exitCode=${danger.exitCode}, 含 rejected: ${String(danger.output).includes('rejected')}`);
  } catch (err) {
    console.error('\n❌ 测试中断:', err);
    results.push({ name: '整体测试', pass: false, detail: String(err) });
  } finally {
    await client.close();
    console.log('\n🧹 客户端已关闭');
  }

  const passed = results.filter((r) => r.pass).length;
  console.log('\n' + '═'.repeat(60));
  console.log('📊 测试汇总:');
  results.forEach((r) => {
    console.log(`   ${r.pass ? '✅' : '❌'} ${r.name}: ${r.detail}`);
  });
  console.log(`\n   通过 ${passed}/${results.length}`);
  console.log('═'.repeat(60));
}

main().catch(console.error);
