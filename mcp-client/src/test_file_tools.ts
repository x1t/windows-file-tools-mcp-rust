/**
 * MCP File Tools 综合测试
 *
 * 验证 file-bash-tools-mcp.exe 暴露的 6 个工具:
 *   write_file / read_file / edit_file / glob / grep / TodoWrite
 *
 * 服务器: target/release/file-bash-tools-mcp.exe
 * 运行:   node dist/test_file_tools.js
 */
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';
import path from 'path';
import { fileURLToPath } from 'url';
import { rm, mkdir, readFile } from 'fs/promises';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SERVER_PATH = path.resolve(__dirname, '../../target/release/file-bash-tools-mcp.exe');
const TEST_BASE = path.resolve(__dirname, '../../test_files/mcp_e2e');
const SRC_DIR = path.resolve(__dirname, '../../src');
const MCP_CLIENT_SRC = path.resolve(__dirname, '..', 'src');

const results: { name: string; pass: boolean; detail: string }[] = [];

function report(name: string, pass: boolean, detail: string): void {
  results.push({ name, pass, detail });
  console.log(`\n${pass ? '✅' : '❌'} [${name}] ${pass ? 'PASS' : 'FAIL'}`);
  console.log(`   ${detail}`);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

interface ToolReply {
  text: string;
  isError: boolean;
}

async function callText(client: Client, name: string, args: Record<string, unknown>): Promise<ToolReply> {
  const result = await client.callTool({ name, arguments: args });
  const content = result.content as { type: string; text: string }[];
  const text = content && content.length > 0 ? content[0].text : '';
  return { text, isError: (result as { isError?: boolean }).isError === true };
}

async function expectError(
  client: Client,
  name: string,
  args: Record<string, unknown>,
): Promise<{ pass: boolean; detail: string }> {
  try {
    const r = await callText(client, name, args);
    if (r.isError) {
      return { pass: true, detail: `返回 isError=true: ${r.text.slice(0, 100)}` };
    }
    return { pass: false, detail: `未报错，返回: ${r.text.slice(0, 140)}` };
  } catch (e) {
    return { pass: true, detail: `抛出异常: ${e instanceof Error ? e.message : String(e)}` };
  }
}

function extractInt(text: string, label: string): number | null {
  const m = text.match(new RegExp(`${label}:\\s*(\\d+)`));
  return m ? Number(m[1]) : null;
}

async function main(): Promise<void> {
  console.log('='.repeat(64));
  console.log('  MCP File Tools 综合测试 (6 个工具)');
  console.log('='.repeat(64));
  console.log(`服务器: ${SERVER_PATH}\n`);

  // 准备干净的测试目录（真实文件操作，非 mock）
  await rm(TEST_BASE, { recursive: true, force: true });
  await mkdir(TEST_BASE, { recursive: true });

  const client = new Client({ name: 'test-file-tools', version: '1.0.0' }, { capabilities: {} });
  const transport = new StdioClientTransport({ command: SERVER_PATH, args: [] });

  try {
    await client.connect(transport);
    console.log('✅ 连接成功\n');

    // ━━━ 步骤 0: 工具清单 ━━━
    console.log('━'.repeat(64));
    console.log('📋 步骤 0: tools/list 校验 6 个工具');
    const tools = await client.listTools();
    const names = tools.tools.map((t) => t.name);
    const expected = ['write_file', 'read_file', 'edit_file', 'glob', 'grep', 'TodoWrite'];
    const missing = expected.filter((n) => !names.includes(n));
    const extra = names.filter((n) => !expected.includes(n));
    console.log(`   暴露工具(${names.length}): ${names.join(', ')}`);
    report(
      '工具清单',
      names.length === expected.length && missing.length === 0 && extra.length === 0,
      `期望 ${expected.length} 个，实际 ${names.length} 个；缺失=${missing.join(',') || '无'}，多余=${extra.join(',') || '无'}`,
    );

    // ━━━ 步骤 1: write_file ━━━
    console.log('\n' + '━'.repeat(64));
    console.log('📝 步骤 1: write_file 原子写入');
    const w1Path = path.join(TEST_BASE, 'write_test.txt');
    const w1Content = 'Hello World\n这是第二行中文\nhello again\n最后一行\n';
    const w1 = await callText(client, 'write_file', { file_path: w1Path, content: w1Content });
    const w1Bytes = extractInt(w1.text, '字节数');
    console.log(`   -> ${w1.text}`);
    report(
      'write_file 新建文件',
      !w1.isError && w1Bytes === Buffer.byteLength(w1Content),
      `isError=${w1.isError}, 字节数=${w1Bytes}, 期望=${Buffer.byteLength(w1Content)}`,
    );

    const disk1 = await readFile(w1Path, 'utf8');
    report('write_file 落盘内容一致', disk1 === w1Content, `磁盘 ${disk1.length} 字节, 期望 ${w1Content.length} 字节`);

    const w2Path = path.join(TEST_BASE, 'nested', 'deep', 'auto.txt');
    const w2 = await callText(client, 'write_file', { file_path: w2Path, content: 'auto dir' });
    report('write_file 自动创建嵌套目录', !w2.isError, w2.text.slice(0, 100));

    const w3 = await callText(client, 'write_file', { file_path: w1Path, content: 'OVERWRITTEN' });
    const disk3 = await readFile(w1Path, 'utf8');
    report('write_file 覆盖已有文件', !w3.isError && disk3 === 'OVERWRITTEN', `覆盖后磁盘内容=${disk3}`);

    // ━━━ 步骤 2: read_file ━━━
    console.log('\n' + '━'.repeat(64));
    console.log('📖 步骤 2: read_file');
    const r1 = await callText(client, 'read_file', { file_path: w1Path });
    const r1Total = extractInt(r1.text, '总计行数');
    console.log(`   读覆盖后文件(单行) -> ${r1.text.split('\n')[0]}`);
    report('read_file 读全文', !r1.isError && r1Total === 1, `总计行数=${r1Total}（文件当前 1 行）`);

    const multiPath = path.join(TEST_BASE, 'multi.txt');
    const multiContent = 'line1\nline2\nline3\nline4\nline5\n';
    await callText(client, 'write_file', { file_path: multiPath, content: multiContent });
    const r2 = await callText(client, 'read_file', { file_path: multiPath, offset: 2, limit: 2 });
    const r2Returned = extractInt(r2.text, '返回行数');
    const r2HasLine2 = r2.text.includes('2\tline2');
    const r2HasLine3 = r2.text.includes('3\tline3');
    report(
      'read_file offset/limit 分页',
      !r2.isError && r2Returned === 2 && r2HasLine2 && r2HasLine3,
      `返回行数=${r2Returned}, 含line2=${r2HasLine2}, 含line3=${r2HasLine3}`,
    );

    const e1 = await expectError(client, 'read_file', { file_path: path.join(TEST_BASE, 'nope_missing.txt') });
    report('read_file 不存在文件报错', e1.pass, e1.detail);

    // ━━━ 步骤 3: edit_file ━━━
    console.log('\n' + '━'.repeat(64));
    console.log('✏️  步骤 3: edit_file 原子替换');
    const edPath = path.join(TEST_BASE, 'edit.txt');
    const edContent = 'hello world\nhello again\nHELLO caps\n';
    await callText(client, 'write_file', { file_path: edPath, content: edContent });

    const ed1 = await callText(client, 'edit_file', {
      file_path: edPath,
      old_string: 'hello',
      new_string: 'hello!',
      replace_all: false,
    });
    const ed1Count = extractInt(ed1.text, '替换次数');
    const diskEd1 = await readFile(edPath, 'utf8');
    report(
      'edit_file 单次替换',
      !ed1.isError && ed1Count === 1 && diskEd1.startsWith('hello! world'),
      `替换次数=${ed1Count}, 磁盘首行=${JSON.stringify(diskEd1.split('\n')[0])}`,
    );

    const ed2 = await callText(client, 'edit_file', {
      file_path: edPath,
      old_string: 'hello',
      new_string: 'hi',
      replace_all: true,
    });
    const ed2Count = extractInt(ed2.text, '替换次数');
    const diskEd2 = await readFile(edPath, 'utf8');
    report(
      'edit_file replace_all',
      !ed2.isError && ed2Count === 2 && diskEd2.includes('hi! world') && diskEd2.includes('hi again'),
      `替换次数=${ed2Count}, 磁盘含 hi! world=${diskEd2.includes('hi! world')}, 含 hi again=${diskEd2.includes('hi again')}`,
    );

    const ed3 = await callText(client, 'edit_file', {
      file_path: edPath,
      old_string: 'NO_SUCH_TEXT',
      new_string: 'Y',
      replace_all: false,
    });
    report('edit_file 无匹配提示', !ed3.isError && ed3.text.includes('未找到'), ed3.text.slice(0, 80));

    // ━━━ 步骤 4: glob ━━━
    console.log('\n' + '━'.repeat(64));
    console.log('🔍 步骤 4: glob 文件匹配');
    const g1 = await callText(client, 'glob', { pattern: '*.rs', path: SRC_DIR });
    const g1Count = extractInt(g1.text, '匹配数');
    const g1Lines = g1.text.split('\n').filter((l) => /^\d+\. /.test(l));
    const g1AllRs = g1Lines.length === (g1Count ?? 0) && g1Lines.every((l) => l.trim().endsWith('.rs'));
    console.log(`   *.rs 于 ${SRC_DIR} -> 匹配数=${g1Count}`);
    report('glob 简单模式', !g1.isError && (g1Count ?? 0) > 0 && g1AllRs, `匹配数=${g1Count}, 全部 .rs=${g1AllRs}`);

    const g2 = await callText(client, 'glob', { pattern: '**/*.ts', path: MCP_CLIENT_SRC });
    const g2Count = extractInt(g2.text, '匹配数');
    console.log(`   **/*.ts 于 mcp-client/src -> 匹配数=${g2Count}`);
    report('glob 递归模式', !g2.isError && (g2Count ?? 0) > 0, `匹配数=${g2Count}`);

    const g3 = await expectError(client, 'glob', { pattern: '*.txt', path: path.join(TEST_BASE, 'no_such_dir') });
    report('glob 路径不存在报错', g3.pass, g3.detail);

    // ━━━ 步骤 5: grep ━━━
    console.log('\n' + '━'.repeat(64));
    console.log('🧪 步骤 5: grep 三模式');
    const gp1 = await callText(client, 'grep', {
      pattern: 'async fn',
      path: SRC_DIR,
      output_mode: 'content',
      show_line_numbers: true,
    });
    const gp1Total = extractInt(gp1.text, '总匹配数');
    console.log(`   content "async fn" -> 总匹配数=${gp1Total}`);
    report(
      'grep content 模式',
      !gp1.isError && (gp1Total ?? 0) > 0 && gp1.text.includes('async fn'),
      `总匹配数=${gp1Total}, 含匹配=${gp1.text.includes('async fn')}`,
    );

    const gp2 = await callText(client, 'grep', {
      pattern: 'tokio',
      path: SRC_DIR,
      output_mode: 'files_with_matches',
    });
    const gp2Files = extractInt(gp2.text, '匹配文件数');
    console.log(`   files_with_matches "tokio" -> 匹配文件数=${gp2Files}`);
    report('grep files_with_matches 模式', !gp2.isError && (gp2Files ?? 0) > 0, `匹配文件数=${gp2Files}`);

    const libFile = path.join(SRC_DIR, 'lib.rs');
    const gp3 = await callText(client, 'grep', { pattern: 'pub', path: libFile, output_mode: 'count' });
    const gp3Total = extractInt(gp3.text, '总匹配数');
    console.log(`   count "pub" in lib.rs -> 总匹配数=${gp3Total}`);
    report('grep count 模式', !gp3.isError && (gp3Total ?? 0) > 0, `总匹配数=${gp3Total}`);

    const gp4 = await expectError(client, 'grep', { pattern: '[', path: SRC_DIR, output_mode: 'content' });
    report('grep 非法正则报错', gp4.pass, gp4.detail);

    const gp5 = await expectError(client, 'grep', { pattern: 'x', path: SRC_DIR, output_mode: 'bogus' });
    report('grep 非法输出模式报错', gp5.pass, gp5.detail);

    // ━━━ 步骤 6: TodoWrite ━━━
    console.log('\n' + '━'.repeat(64));
    console.log('📋 步骤 6: TodoWrite 任务清单');
    const t1 = await callText(client, 'TodoWrite', {
      todos: [{ content: '测试待办', status: 'pending', active_form: '等待处理' }],
    });
    const t1Total = extractInt(t1.text, '总任务数');
    report('TodoWrite 单条', !t1.isError && t1Total === 1 && t1.text.includes('待处理: 1'), `总任务数=${t1Total}`);

    const t2 = await callText(client, 'TodoWrite', {
      todos: [
        { content: '任务A', status: 'pending', active_form: '等待' },
        { content: '任务B', status: 'in_progress', active_form: '执行中' },
        { content: '任务C', status: 'completed', active_form: '已完成' },
      ],
    });
    const t2Total = extractInt(t2.text, '总任务数');
    const t2Ok =
      t2.text.includes('待处理: 1') && t2.text.includes('进行中: 1') && t2.text.includes('已完成: 1');
    report('TodoWrite 多条混合状态', !t2.isError && t2Total === 3 && t2Ok, `总任务数=${t2Total}, 状态统计=${t2Ok}`);

    await sleep(200);
  } catch (err) {
    console.error('\n❌ 测试中断:', err);
    results.push({ name: '整体测试', pass: false, detail: String(err) });
  } finally {
    await client.close();
    await rm(TEST_BASE, { recursive: true, force: true }).catch(() => undefined);
    console.log('\n🧹 测试产物已清理');
  }

  const passed = results.filter((r) => r.pass).length;
  console.log('\n' + '═'.repeat(64));
  console.log('📊 测试汇总:');
  results.forEach((r) => {
    console.log(`   ${r.pass ? '✅' : '❌'} ${r.name}: ${r.detail}`);
  });
  console.log(`\n   通过 ${passed}/${results.length}`);
  console.log('═'.repeat(64));
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
