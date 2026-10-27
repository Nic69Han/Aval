"""Shared helpers for the eval-harness conformance tests (no scenario header: not a test)."""
import hashlib, json, os, re, shlex, subprocess, sys, tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
HARNESS = os.path.join(REPO, "experiments", "e2-in-context", "harness.py")
FIX = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures")


def fail(msg):
    print("FAIL: " + msg)
    sys.exit(1)


def check(cond, msg):
    if not cond:
        fail(msg)


def aval_bin():
    b = os.environ.get("AVAL_BIN")
    if b and os.path.exists(shlex.split(b)[0]):
        return shlex.split(b)[0]
    p = os.path.join(REPO, "target", "debug", "aval")
    if not os.path.exists(p):
        subprocess.run(["cargo", "build", "-q", "-p", "aval-cli"], cwd=REPO, check=True)
    return p


def skill_text():
    r = subprocess.run([aval_bin(), "skill", "print"], capture_output=True)
    check(r.returncode == 0, "aval skill print failed")
    return r.stdout


def harness(args, timeout=900, cwd=None, env=None):
    e = dict(os.environ)
    e.update(env or {})
    return subprocess.run([sys.executable, HARNESS] + args, capture_output=True, text=True,
                          timeout=timeout, cwd=cwd or REPO, env=e)


def run(args, out, timeout=900, env=None, expect_exit=0):
    """Run `harness.py run ... --out out`; fail the test on a harness error (exit != 0)."""
    cmd = ["run"] + ([] if "--aval-bin" in args else ["--aval-bin", aval_bin()]) + args + ["--out", out]
    r = harness(cmd, timeout=timeout, env=env)
    check(r.returncode == expect_exit, "harness exited %d\nstdout: %s\nstderr: %s" % (r.returncode, r.stdout[-800:], r.stderr[-800:]))
    return r


def results(out):
    return json.load(open(os.path.join(out, "results.json")))


def fx(name):
    return os.path.join(FIX, name)


def tmp():
    return tempfile.TemporaryDirectory()


def recorder_cmd(record_dir, answers_dir):
    return "%s %s %s %s" % (shlex.quote(sys.executable), shlex.quote(fx("recorder.py")),
                            shlex.quote(record_dir), shlex.quote(fx(answers_dir)))


def prompts(record_dir):
    files = sorted(f for f in os.listdir(record_dir) if f.startswith("prompt_"))
    return [open(os.path.join(record_dir, f), encoding="utf-8").read() for f in files]


MBPP_REVISION = "b2d74c91837c3f2a20c1299ae98133cbe7cfa077"
MBPP_SHA256 = "dc20030b3788fccf617444edcb34138ef13d7e4fafd17bfcb8c1279dbb12399b"
MBPP_URL = ("https://huggingface.co/datasets/evalplus/mbppplus/resolve/%s/data/test-00000-of-00001-d5781c9c51e02795.parquet"
            % MBPP_REVISION)


def fetch_mbpp_parquet():
    """Path of the pinned MBPP+ parquet file, downloaded once into $AVAL_E2_CACHE (default: the system temp dir).

    A network failure fails the test with a NETWORK message; a hash mismatch (cached or downloaded file) fails
    with DATASET CHANGED."""
    import ssl, urllib.request
    cache = os.environ.get("AVAL_E2_CACHE") or os.path.join(tempfile.gettempdir(), "aval-e2-cache")
    os.makedirs(cache, exist_ok=True)
    path = os.path.join(cache, "mbppplus-%s.parquet" % MBPP_SHA256[:12])
    if os.path.exists(path) and hashlib.sha256(open(path, "rb").read()).hexdigest() == MBPP_SHA256:
        return path
    ctx = ssl.create_default_context()
    for cand in (os.environ.get("SSL_CERT_FILE"), os.environ.get("REQUESTS_CA_BUNDLE"), "/root/.ccr/ca-bundle.crt"):
        if cand and os.path.exists(cand):
            ctx = ssl.create_default_context(cafile=cand)
            break
    try:
        with urllib.request.urlopen(MBPP_URL, timeout=300, context=ctx) as resp:
            data = resp.read()
    except Exception as e:
        fail("NETWORK: cannot download %s (this test needs network access, or a populated $AVAL_E2_CACHE, and is not "
             "skipped): %r" % (MBPP_URL, e))
    digest = hashlib.sha256(data).hexdigest()
    check(digest == MBPP_SHA256, "DATASET CHANGED: SHA-256 %s, expected %s (revision %s)" % (digest, MBPP_SHA256, MBPP_REVISION))
    tmpname = path + ".part"
    open(tmpname, "wb").write(data)
    os.replace(tmpname, path)
    return path
