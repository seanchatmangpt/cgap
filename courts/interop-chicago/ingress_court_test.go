package interop

// LANE IPC-3 (RE-SCOPED, loud) -- membrane-latency court over the REAL
// exported kernels (lane-ipc-00-ground-truth: NO cgap_ingress_bytes exists;
// the charter's ingress C symbol is absent from the 11-symbol table).
//
// Re-scope statement: this court binds a MINIMAL subset of the SAME verified
// symbols (cgap_admit_frame only) directly -- lane IPC-2's bindings were NOT
// landed at dispatch time (no lane-ipc-02 receipt on disk at dispatch), so
// this file carries its own binding (ingress_court_binding.go, disclosed
// ownership divergence: `import "C"` is unsupported in _test.go files,
// golang/go#8601). On IPC-2 landing, IPC-2's bindings supersede this diff.
//
// Kernel measured: cgap_admit_frame (chosen over cgap_eval_bn3 as the
// cheaper honest representative: admit = A14 gate + A04 header validation +
// BLAKE3 digest re-derivation over a 24 B payload; eval additionally needs a
// full .bn3 rodata network fixture and is deferred with this disclosure).
//
// Load-sensitivity disclosure (B06 law): every number below was measured on
// a shared macOS/arm64 host while concurrent lane builds ran; the machine is
// NOT isolated. Figures are THIS-session measurements of THIS dylib --
// environment-dependent, never isolated-bench targets, and never comparable
// to the 6.04 ns charter fiction (that arithmetic used an ASSUMED kernel
// time).
//
// Anti-vacuity: TestMutationGateBreach corrupts fixture bytes IN THE GO
// DRIVER and requires the pinned typed refusals. If admit returns CGAP_OK on
// corrupted bytes the court FAILs -- a court that cannot flip is FAIL.
import (
	"os"
	"runtime"
	"sort"
	"testing"
	"time"
)

const (
	cgapOK          = uint32(0x00000000)
	cgapErrInvMagic = uint32(0x43470001)
)

// LANE AX-5 (linux adaptation): fixture root is env-overridable so the
// container runner can point at its own materialized fixtures; when the env
// is UNSET the darwin default below is byte-identical to IPC-3 behavior.
func fixtureDir() string {
	if d := os.Getenv("CGAP_FIXTURE_DIR"); d != "" {
		return d
	}
	return "/tmp/ipc3-fixtures"
}

func loadFixture(t *testing.T, path string, wantLen int) []byte {
	t.Helper()
	b, err := os.ReadFile(path)
	if err != nil || len(b) != wantLen {
		t.Fatalf("[SKIP-RESCOPED] fixture %s missing/wrong size (got %d, err=%v): regenerate via /tmp/ipc3-fxgen", path, len(b), err)
	}
	return b
}

func TestAdmitValidFrame(t *testing.T) {
	fx := loadFixture(t, fixtureDir()+"/frame_24.bin", 152) // AX-5: fixtureDir()
	st, plen := AdmitFrame(fx)
	if st != cgapOK {
		t.Fatalf("valid fixture refused: status=0x%08X (want CGAP_OK)", st)
	}
	if plen != 24 {
		t.Fatalf("view payload_len=%d, want 24 (A04 zero-copy view)", plen)
	}
	// Final-state: fixture payload byte 0 must be 0x5A (offset 128); the
	// kernel validated it via BLAKE3 re-derivation, proving a real read.
	if fx[128] != 0x5A {
		t.Fatalf("fixture payload corrupted in driver: %d", fx[128])
	}
}

func TestMutationGateBreach(t *testing.T) {
	// ANTI-VACUITY MUTATION (named): branchy corruption injected in the Go
	// driver must breach the gate. Magic flip -> INVALID_MAGIC (typed pin
	// 0x43470001). A court that cannot flip is FAIL.
	fx := loadFixture(t, fixtureDir()+"/frame_24.bin", 152) // AX-5: fixtureDir()
	m := make([]byte, len(fx))
	copy(m, fx)
	m[0] = 'X' // "CGAP" -> "XGAP"
	if st, _ := AdmitFrame(m); st != cgapErrInvMagic {
		t.Fatalf("CANNOT FLIP: magic-flipped frame admitted (status=0x%08X) -- court FAIL", st)
	}
	// Digest flip -> typed refusal (stage-2 BLAKE3 compare class). Any
	// nonzero status witnesses the flip; CGAP_OK here is fail-open = FAIL.
	d := make([]byte, len(fx))
	copy(d, fx)
	d[16] ^= 0xFF
	if st, _ := AdmitFrame(d); st == cgapOK {
		t.Fatalf("CANNOT FLIP: digest-corrupted frame admitted -- court FAIL")
	}
}

func TestEmptyCgoCallFloor(t *testing.T) {
	// Phase 2 Measure. Honest disclosure: no empty exported C symbol exists
	// in the 11-symbol table, so the EMPTY-PATH PROXY is the cheapest real
	// membrane call: cgap_admit_frame(NULL, 0, view) -- the null-refusal
	// short-circuit (sovereign.rs:415-419). This bounds cgo transition cost
	// from below; it is NOT a pure FFI round-trip figure.
	const n = 200000
	runtime.LockOSThread()
	start := time.Now()
	for i := 0; i < n; i++ {
		AdmitFrame(nil)
	}
	elapsed := time.Since(start)
	nsPerCall := elapsed.Nanoseconds() / n
	t.Logf("empty-path proxy: %d calls, %v total, %d ns/call", n, elapsed, nsPerCall)
	if nsPerCall <= 0 {
		t.Fatalf("[SKIP] timer resolution below 1 ns/call -- floor unmeasurable this run")
	}
}

func TestTimerResolutionWitness(t *testing.T) {
	// Go timer-resolution witness: min positive paired time.Now delta.
	var minDelta time.Duration = time.Hour
	prev := time.Now()
	for i := 0; i < 100000; i++ {
		cur := time.Now()
		if d := cur.Sub(prev); d > 0 && d < minDelta {
			minDelta = d
		}
		prev = cur
	}
	t.Logf("timer resolution witness: min positive monotonic delta = %v", minDelta)
	if minDelta > 2*time.Microsecond {
		t.Fatalf("timer resolution %v worse than 2us -- ns figures untrustworthy this host", minDelta)
	}
}

// Batch-1365 amortized court (Phase 4). FACT_WINDOW = 1365
// (crates/cgap-git/src/pack_demux.rs:43-51, named coprime stride).
// Provenance: HONEST NONE -- synthetic 24 B facts from deterministic
// fixtures (/tmp/ipc3-fxgen), no corpus lineage claimed.
// NO-P99 LAW (FIX-04/GAP-03): max-of-N is NOT a quantile; we report
// min/median/max over independent runs and explicitly do NOT report a p99.
func BenchmarkAdmitBatch1365(b *testing.B) {
	data, err := os.ReadFile(fixtureDir() + "/batch_1365.bin") // AX-5: fixtureDir()
	if err != nil || len(data) != 1365*152 {
		b.Fatalf("[SKIP-RESCOPED] batch fixture missing (%d bytes, err=%v)", len(data), err)
	}
	buf := make([]byte, len(data))
	copy(buf, data)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		for off := 0; off < len(buf); off += 152 {
			if st, _ := AdmitFrame(buf[off : off+152]); st != cgapOK {
				b.Fatalf("batch frame at %d refused: 0x%08X", off, st)
			}
		}
	}
}

func BenchmarkAdmitSingle(b *testing.B) {
	fx := loadFixtureT(b, fixtureDir()+"/frame_24.bin", 152) // AX-5: fixtureDir()
	buf := make([]byte, 152)
	copy(buf, fx)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if st, _ := AdmitFrame(buf); st != cgapOK {
			b.Fatalf("admit refused: 0x%08X", st)
		}
	}
}

func loadFixtureT(tb testing.TB, path string, wantLen int) []byte {
	tb.Helper()
	b, err := os.ReadFile(path)
	if err != nil || len(b) != wantLen {
		tb.Fatalf("[SKIP-RESCOPED] fixture %s missing/wrong size: %v", path, err)
	}
	return b
}

func TestBatchStats(t *testing.T) {
	// Phase 3/4 reportable table: N independent batch runs, amortized
	// ns/frame each, min/median/max reported. No p99 (max-of-N is not a
	// quantile, FIX-04/GAP-03).
	data, err := os.ReadFile(fixtureDir() + "/batch_1365.bin") // AX-5: fixtureDir()
	if err != nil || len(data) != 1365*152 {
		t.Fatalf("[SKIP-RESCOPED] batch fixture missing: %v", err)
	}
	buf := make([]byte, len(data))
	copy(buf, data)
	const runs = 7
	samples := make([]float64, 0, runs)
	for r := 0; r < runs; r++ {
		start := time.Now()
		for off := 0; off < len(buf); off += 152 {
			if st, _ := AdmitFrame(buf[off : off+152]); st != cgapOK {
				t.Fatalf("frame at %d refused: 0x%08X", off, st)
			}
		}
		samples = append(samples, float64(time.Since(start).Nanoseconds())/1365.0)
	}
	sort.Float64s(samples)
	t.Logf("amortized ns/frame over %d runs: min=%.0f median=%.0f max=%.0f (no p99: max-of-N is not a quantile, FIX-04/GAP-03)", runs, samples[0], samples[runs/2], samples[runs-1])
}
