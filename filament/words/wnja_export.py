# Japanese WordNet (NICT, wnjpn.db v1.1) cut to what Motolii's search needs: each Japanese word, its kana as
# hiragana, and the English words of the same synsets, most frequent first. Run: python3 -I wnja_export.py <wnjpn.db> <out.tsv>
import sqlite3, sys, collections
db = sqlite3.connect(sys.argv[1])
rows = db.execute("""select j.lemma, e.lemma, coalesce(se.freq, 0) + coalesce(sj.freq, 0) from word j
  join sense sj on sj.wordid = j.wordid join sense se on se.synset = sj.synset and se.lang = 'eng'
  join word e on e.wordid = se.wordid where j.lang = 'jpn'""")
def hira(s): return ''.join(chr(ord(c) - 0x60) if 'ァ' <= c <= 'ヶ' else c for c in s)
words = collections.defaultdict(dict)
for ja, en, f in rows:
    en = en.replace('_', ' ')
    k = hira(ja)
    words[k][en] = max(words[k].get(en, 0), f)
with open(sys.argv[2], 'w', encoding='utf-8') as out:
    for ja in sorted(words):
        out.write(ja + '\t' + ','.join(sorted(words[ja], key=lambda e: (-words[ja][e], e))) + '\n')
