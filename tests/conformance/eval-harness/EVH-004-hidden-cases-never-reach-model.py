#!/usr/bin/env python3
# scenario: EVH-004
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _common import *

task = json.load(open(os.path.join(REPO, "experiments", "e2-in-context", "tasks", "HE-025.json")))
skill = skill_text().decode("utf-8")
answers = "".join(open(os.path.join(FIX, "recorder_answers", f)).read() for f in ("1.txt", "2.txt"))
# text the model itself wrote (its own examples come back in diagnostics) is not a leak
allowed = re.sub(r"\s", "", skill + task["statement"] + task["rust_signature"] + task["aval"]["name"] + answers)
forbidden = set()
for h in task["hidden"]:
    for v in h["args"] + [h["expected"]]:
        for s in (json.dumps(v), json.dumps(v, separators=(",", ":"))):
            if len(s) >= 4 and re.sub(r"\s", "", s) not in allowed:
                forbidden.add(s)
    forbidden.add("%s(%s)" % (task["aval"]["name"], ", ".join(json.dumps(a) for a in h["args"])))
check("185193" in forbidden and "[3, 3, 19, 19]" in forbidden, "test setup: distinctive hidden values expected")

def scan(text, where):
    for s in forbidden:
        check(s not in text, "%s contains hidden text %r" % (where, s))

# 1. replay: every prompt recorded in results.json (the first answer is wrong, so a repair prompt exists)
with tmp() as out:
    run(["--provider", "replay", "--responses", fx("hidden_secrecy.json"), "--task", "HE-025", "--langs", "aval"], out)
    att = results(out)["tasks"]["HE-025"]["aval"]["attempts"]
    check(len(att) == 2 and att[0]["failed_cases"], "setup: first answer must fail hidden cases")
    for a in att:
        check(a["prompt"], "empty recorded prompt")
        scan(a["prompt"], "recorded prompt of attempt %d" % a["attempt"])
# 2. command provider: the real conversation sent on stdin
with tmp() as out, tmp() as rec:
    run(["--provider", "command", "--command", recorder_cmd(rec, "recorder_answers"), "--task", "HE-025", "--langs", "aval"], out)
    ps = prompts(rec)
    check(len(ps) == 2, "expected 2 prompts, got %d" % len(ps))
    check(task["statement"] in ps[0], "the first prompt must contain the statement (capture sanity)")
    for i, p in enumerate(ps):
        scan(p, "stdin prompt %d" % (i + 1))

# 3. every task, both languages: prompts of a run where every answer lacks code (so every repair prompt is sent)
skill_norm = re.sub(r"\s", "", skill)
tdir = os.path.join(REPO, "experiments", "e2-in-context", "tasks")
with tmp() as out:
    run(["--provider", "replay", "--responses", fx("no_code_all.json")], out)
    res = results(out)
    n = 0
    for tid, arms in res["tasks"].items():
        t = json.load(open(os.path.join(tdir, tid + ".json")))
        ok_text = skill_norm + re.sub(r"\s", "", t["statement"] + t["rust_signature"] + t["aval"]["name"])
        for lang, arm in arms.items():
            check(len(arm["attempts"]) == 4, "%s %s: expected 4 attempts" % (tid, lang))
            for att in arm["attempts"]:
                n += 1
                for h in t["hidden"]:
                    for v in h["args"] + [h["expected"]]:
                        for s in (json.dumps(v), json.dumps(v, separators=(",", ":"))):
                            sn = re.sub(r"\s", "", s)
                            if len(sn) >= 5 and sn not in ok_text:
                                check(sn not in re.sub(r"\s", "", att["prompt"]), "%s %s attempt %d prompt contains hidden value %r" % (tid, lang, att["attempt"], s))
    check(n == 40 * 2 * 4, "expected 320 prompts, scanned %d" % n)
print("ok")
