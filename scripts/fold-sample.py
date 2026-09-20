#!/usr/bin/env python3
"""macOS `sample` の呼び出し木を、持ち主ごとの self / total へ畳む。

手で時計を置かない — 走った関数は全部ここへ出る。関数が増えても、この file は直さない。
Linux は `perf script` の木を同じ形に読み替えれば共通で使える。
"""
import re
import sys
import collections

# 字下げは空白と飾り(`+ ! : |`)。飾りの長さがそのまま深さ。
ROW = re.compile(r'^(?P<ind>[ +!:|]*)(?P<n>\d+) (?P<rest>.*)$')
FRAME = re.compile(r'^(?P<name>.*?)\s+\(in (?P<module>[^)]+)\)')
# 眠っている葉。ここで止まっている標本は仕事ではない。
IDLE = ('__workq_kernreturn', '__psynch_cvwait', 'mach_msg2_trap', 'kevent', 'semaphore_wait_trap',
        '__select', '__read_nocancel', 'poll', '__ulock_wait', 'swtch_pri', 'thread_switch',
        '__sigsuspend_nocancel', 'accept', '__accept_nocancel')


def fold(path):
    threads = collections.OrderedDict()
    cur = None
    stack = []

    def close(to_depth):
        while stack and stack[-1][0] >= to_depth:
            _, key, count, kids, leaf = stack.pop()
            d = threads[cur]
            d['self'][key] += count - kids
            if leaf.startswith(IDLE):
                d['idle'] += count - kids
            if stack:
                dd, k, c, kd, ln = stack[-1]
                stack[-1] = (dd, k, c, kd + count, ln)

    for raw in open(path, errors='replace'):
        m = ROW.match(raw.rstrip('\n'))
        if not m:
            continue
        rest, depth, n = m.group('rest'), len(m.group('ind')), int(m.group('n'))
        if rest.startswith('Thread_'):
            close(-1)
            stack.clear()
            cur = rest.split(':', 1)[1].strip() if ':' in rest else rest.split()[0]
            threads.setdefault(cur, {'self': collections.Counter(), 'total': collections.Counter(), 'idle': 0, 'n': n})
            continue
        f = FRAME.match(rest)
        if not f or cur is None:
            continue
        name = re.sub(r'::h[0-9a-f]{16}$', '', f.group('name')).strip()
        close(depth)
        key = '%s  [%s]' % (name, f.group('module'))
        threads[cur]['total'][key] += n
        stack.append((depth, key, n, 0, name))
    close(-1)
    return threads


def main():
    path, top = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 25
    threads = fold(path)
    busy = sorted(((name, d, d['n'] - d['idle']) for name, d in threads.items()), key=lambda x: -x[2])
    print('\nPROBE room=playback-profile file=%s' % path)
    if all(work <= 0 for _, _, work in busy):
        print('どの thread も眠っていました — 再生していない窓を測っています。')
    print('働いている thread(標本のうち眠っていない分):')
    for name, d, work in busy[:8]:
        print('  %6d/%-6d %s' % (work, d['n'], name))
    for name, d, work in busy[:3]:
        if work <= 0:
            continue
        print('\n== %s   働き=%d/%d 標本' % (name, work, d['n']))
        print('%6s%8s%8s  関数' % ('self%', 'self', 'total'))
        for key, s in d['self'].most_common(top):
            # 眠っている葉は仕事ではないので並べない(分母からも外してある)。
            if s == 0 or key.split('  [')[0].startswith(IDLE):
                continue
            print('%6.1f%8d%8d  %s' % (s * 100.0 / work, s, d['total'][key], key))


if __name__ == '__main__':
    main()
