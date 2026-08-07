import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 服务器路径：编译产物位于 mcp-client/dist，向上两级即项目根目录
const SERVER_PATH = path.resolve(__dirname, '../../dist/bash-tools.exe');

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
  console.log('  MCP Bash Tools 综合测试 (bash / bash_output / kill_shell)');
  console.log('='.repeat(60));
  console.log(`服务器路径: ${SERVER_PATH}\n`);

  const client = new Client({ name: 'test-all-tools', version: '1.0.0' }, { capabilities: {} });
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
    report(
      '工具暴露',
      toolOk,
      `期望 ${expected.join(', ')}, 实际 ${toolNames.join(', ')} (共 ${toolNames.length} 个)`,
    );

    // ========== 步骤 1: bash 前台 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 1: bash 前台执行');
    const fg = await client.callTool({
      name: 'bash',
      arguments: { command: 'Write-Output "HELLO-FROM-FG-$PID"', timeout: 10000, run_in_background: false },
    });
    const fgData = parseResult(fg);
    console.log(`   Output: ${fgData.output}`);
    console.log(`   ExitCode: ${fgData.exitCode}`);
    const fgOk = fgData.exitCode === 0 && String(fgData.output).includes('HELLO-FROM-FG');
    report(
      'bash 前台',
      fgOk,
      `exitCode=${fgData.exitCode}, 输出包含 HELLO-FROM-FG: ${String(fgData.output).includes('HELLO-FROM-FG')}`,
    );

    // ========== 步骤 2: bash 后台 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 2: bash 后台执行 (run_in_background=true)');
    const bg = await client.callTool({
      name: 'bash',
      arguments: {
        command: 'C:\\Windows\\System32\\PING.EXE 127.0.0.1 -t',
        timeout: 60000,
        run_in_background: true,
        description: '后台持续 ping',
      },
    });
    const bgData = parseResult(bg);
    console.log(`   Output: ${bgData.output}`);
    console.log(`   ShellID: ${bgData.shell_id}`);
    const bgOk = typeof bgData.shell_id === 'string' && bgData.shell_id.length > 0;
    report('bash 后台', bgOk, `shell_id=${bgData.shell_id ?? '(空)'}`);

    // ========== 步骤 3: bash_output 增量输出 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 3: bash_output 增量输出 + 状态追踪');
    await sleep(3000);
    const o1 = parseResult(
      await client.callTool({ name: 'bash_output', arguments: { bash_id: bgData.shell_id } }),
    );
    console.log(`   第一次 status=${o1.status}, ping 回复=${countPingReplies(o1.output ?? '')}`);
    await sleep(2000);
    const o2 = parseResult(
      await client.callTool({ name: 'bash_output', arguments: { bash_id: bgData.shell_id } }),
    );
    console.log(`   第二次 status=${o2.status}, ping 回复=${countPingReplies(o2.output ?? '')}`);
    const c1 = countPingReplies(o1.output ?? '');
    const c2 = countPingReplies(o2.output ?? '');
    const outOk = o1.status === 'running' && c1 > 0 && c2 > c1;
    report('bash_output 增量', outOk, `第一次=${c1}条, 第二次=${c2}条, 状态=${o1.status}`);

    // ========== 步骤 4: bash_output 正则过滤 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('📝 步骤 4: bash_output 正则过滤');
    const f1 = parseResult(
      await client.callTool({
        name: 'bash_output',
        arguments: { bash_id: bgData.shell_id, filter: 'Reply|来自' },
      }),
    );
    const filteredLines = (f1.output ?? '').split('\n').filter((l: string) => l.trim().length > 0);
    console.log(`   过滤后行数: ${filteredLines.length}`);
    const filterOk = filteredLines.length > 0 && filteredLines.every((l: string) => /Reply|来自/i.test(l));
    report('bash_output 过滤', filterOk, `过滤后 ${filteredLines.length} 行全部匹配: ${filterOk}`);

    // ========== 步骤 5: kill_shell ==========
    console.log('\n' + '━'.repeat(60));
    console.log('🛑 步骤 5: kill_shell 终止后台任务');
    const kill = parseResult(
      await client.callTool({ name: 'kill_shell', arguments: { shell_id: bgData.shell_id } }),
    );
    console.log(`   Message: ${kill.message}`);
    await sleep(1500);
    const afterPing = await checkProcess('PING.EXE');
    console.log(`   终止后 PING.EXE 进程数: ${afterPing.length}`);
    const killOk = String(kill.message).includes('killed') && afterPing.length === 0;
    report('kill_shell', killOk, `message=${kill.message}, PING 残留=${afterPing.length}`);

    // ========== 步骤 6: 前台超时自动转后台 ==========
    console.log('\n' + '━'.repeat(60));
    console.log('⏱️ 步骤 6: 前台超时自动转后台');
    const to = await client.callTool({
      name: 'bash',
      arguments: { command: 'Start-Sleep -Seconds 8; Write-Output "DONE"', timeout: 1000, run_in_background: false },
    });
    const toData = parseResult(to);
    console.log(`   ShellID: ${toData.shell_id}`);
    console.log(`   Output: ${toData.output}`);
    const toOk = typeof toData.shell_id === 'string' && toData.shell_id.length > 0;
    report('前台超时转后台', toOk, `shell_id=${toData.shell_id ?? '(空)'}`);
    if (toData.shell_id) {
      await client.callTool({ name: 'kill_shell', arguments: { shell_id: toData.shell_id } });
      console.log('   🧹 已清理超时任务');
    }
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
