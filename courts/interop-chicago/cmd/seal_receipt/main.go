package main

// seal_receipt — IPC-6 interop-chicago receipt emitter with honesty self-lint.
//
// HONESTY GATES (self-lint, fail-closed at emit time):
//   SL-001: every quantitative bullet must carry a method disclosure
//           (batch-amortized | machine-dependent | load-sensitive).
//   SL-002: every bullet must cite a receipt path.
//   SL-003: the number a bullet asserts must have a this-session command
//           exit in its input (command non-empty AND exit present >= 0).
//   SL-004 (AX-8 where-it-ran): the receipt must record WHERE it ran —
//           non-empty image_digest (sha256:...), ax_runner_version, and
//           sandbox_class (gVisor | runc). A receipt that cannot say where
//           it ran is refused.
// A refusal is exit 1 with named SL lines. The refusal IS the flip.

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"strings"
)

const version = "seal-receipt/0.2.0 (IPC-6+AX-8)"

// Method keywords that count as a quantitative-claim disclosure.
var methodKeywords = []string{"batch-amortized", "machine-dependent", "load-sensitive"}

type CTQ struct {
	Name        string `json:"name"`
	Value       string `json:"value"`
	Method      string `json:"method"`
	Command     string `json:"command"`
	Exit        *int   `json:"exit"`
	ReceiptPath string `json:"receipt_path"`
}

type Bullet struct {
	Text        string `json:"text"`
	CTQ         string `json:"ctq"`
	ReceiptPath string `json:"receipt_path"`
}

type ReceiptInput struct {
	Host string `json:"host"`
	// SL-004 where-it-ran fields; pointers so ABSENT vs EMPTY are distinct.
	// -fill-where fills only absent fields; an explicitly empty value is
	// still a refusal.
	ImageDigest   *string  `json:"image_digest"`
	RunnerVersion *string  `json:"ax_runner_version"`
	SandboxClass  *string  `json:"sandbox_class"`
	GoVersion     string   `json:"go_version"`
	CGO           string   `json:"cgo"`
	Thermal       string   `json:"thermal_caveat"`
	CTQs          []CTQ    `json:"ctqs"`
	Bullets       []Bullet `json:"bullets"`
}

func strv(p *string) string {
	if p == nil {
		return ""
	}
	return *p
}

func hasMethodDisclosure(s string) bool {
	l := strings.ToLower(s)
	for _, k := range methodKeywords {
		if strings.Contains(l, k) {
			return true
		}
	}
	return false
}

func isQuantitative(s string) bool {
	for _, r := range s {
		if r >= '0' && r <= '9' {
			return true
		}
	}
	return false
}

// selfLint returns named refusal lines; empty slice = clean.
func selfLint(in *ReceiptInput) []string {
	var ref []string
	// SL-004: where-it-ran gate (receipt-level, not per-bullet).
	id := strv(in.ImageDigest)
	if !strings.HasPrefix(id, "sha256:") || len(id) != 71 {
		ref = append(ref, "SL-004: image_digest missing or not a sha256: digest (64 hex chars after the prefix)")
	}
	if strv(in.RunnerVersion) == "" {
		ref = append(ref, "SL-004: ax_runner_version missing — the receipt cannot say which AX runner sealed it")
	}
	sc := strings.ToLower(strv(in.SandboxClass))
	if sc != "gvisor" && sc != "runc" {
		ref = append(ref, "SL-004: sandbox_class must be gVisor or runc, got "+fmt.Sprintf("%q", strv(in.SandboxClass)))
	}
	byName := make(map[string]*CTQ, len(in.CTQs))
	for i := range in.CTQs {
		byName[in.CTQs[i].Name] = &in.CTQs[i]
	}
	for _, b := range in.Bullets {
		quoted := fmt.Sprintf("%q", b.Text)
		ctq, ok := byName[b.CTQ]
		if !ok {
			ref = append(ref, "SL-003: bullet "+quoted+" cites CTQ "+fmt.Sprintf("%q", b.CTQ)+" which has no measured row")
			continue
		}
		if isQuantitative(b.Text) {
			if !hasMethodDisclosure(ctq.Method) && !hasMethodDisclosure(b.Text) {
				ref = append(ref, "SL-001: bullet "+quoted+" is quantitative but lacks method disclosure (batch-amortized | machine-dependent | load-sensitive)")
			}
		}
		if strings.TrimSpace(b.ReceiptPath) == "" && strings.TrimSpace(ctq.ReceiptPath) == "" {
			ref = append(ref, "SL-002: bullet "+quoted+" cites no receipt path")
		}
		if ctq.Command == "" || ctq.Exit == nil || *ctq.Exit < 0 {
			ref = append(ref, "SL-003: bullet "+quoted+" number lacks a this-session command exit in its input")
		}
	}
	return ref
}

func emit(in *ReceiptInput) ([]byte, error) {
	var b bytes.Buffer
	fmt.Fprintf(&b, "# receipt-interop-chicago-v1\n\n")
	fmt.Fprintf(&b, "sealed-by: %s\n\n", version)
	fmt.Fprintf(&b, "## Environment\n\n")
	fmt.Fprintf(&b, "- host: %s\n", in.Host)
	fmt.Fprintf(&b, "- image digest: %s\n", strv(in.ImageDigest))
	fmt.Fprintf(&b, "- ax runner version: %s\n", strv(in.RunnerVersion))
	fmt.Fprintf(&b, "- sandbox class: %s\n", strv(in.SandboxClass))
	fmt.Fprintf(&b, "- go: %s\n", in.GoVersion)
	fmt.Fprintf(&b, "- cgo: %s\n", in.CGO)
	fmt.Fprintf(&b, "- thermal caveat: %s\n", in.Thermal)
	fmt.Fprintf(&b, "\n## Per-CTQ measured table\n\n")
	fmt.Fprintf(&b, "| CTQ | measured | method | command | exit | receipt |\n")
	fmt.Fprintf(&b, "|---|---|---|---|---|---|\n")
	for _, c := range in.CTQs {
		exit := "ABSENT"
		if c.Exit != nil {
			exit = fmt.Sprintf("%d", *c.Exit)
		}
		fmt.Fprintf(&b, "| %s | %s | %s | `%s` | %s | %s |\n",
			c.Name, c.Value, c.Method, c.Command, exit, c.ReceiptPath)
	}
	fmt.Fprintf(&b, "\n## Sales bullets (self-linted)\n\n")
	for _, bl := range in.Bullets {
		fmt.Fprintf(&b, "- %s [receipt: %s]\n", bl.Text, bl.ReceiptPath)
	}
	fmt.Fprintf(&b, "\n## Transcript root\n\n")
	h := sha256.Sum256(bytes.TrimSpace(b.Bytes()))
	fmt.Fprintf(&b, "transcript-root (sha256 over the receipt bytes above, trailing whitespace trimmed): %s\n", hex.EncodeToString(h[:]))
	fmt.Fprintf(&b, "DIVERGENCE-NAMED: transcript root is sha256, not BLAKE3 — Go stdlib has no BLAKE3; per dispatch this is named, not faked.\n")
	return b.Bytes(), nil
}

func main() {
	inPath := flag.String("in", "", "input receipt JSON")
	outPath := flag.String("out", "", "output markdown receipt")
	fillWhere := flag.Bool("fill-where", false,
		"fill SL-004 fields from -ax-* flags (container runner); lint still applies to the RESULT")
	axImageDigest := flag.String("ax-image-digest", os.Getenv("AX_IMAGE_DIGEST"), "image digest (SL-004)")
	axRunnerVersion := flag.String("ax-runner-version", os.Getenv("AX_RUNNER_VERSION"), "AX runner version (SL-004)")
	axSandboxClass := flag.String("ax-sandbox-class", os.Getenv("AX_SANDBOX_CLASS"), "sandbox class: gVisor|runc (SL-004)")
	flag.Parse()
	if *inPath == "" || *outPath == "" {
		fmt.Fprintln(os.Stderr, "usage: seal_receipt -in <input.json> -out <output.md>")
		os.Exit(2)
	}
	raw, err := os.ReadFile(*inPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "REFUSED: cannot read input: %v\n", err)
		os.Exit(2)
	}
	var in ReceiptInput
	if err := json.Unmarshal(raw, &in); err != nil {
		fmt.Fprintf(os.Stderr, "REFUSED: malformed input JSON: %v\n", err)
		os.Exit(2)
	}
	if *fillWhere {
		// Fill only ABSENT fields; an explicitly empty value stays and refuses.
		if in.ImageDigest == nil {
			in.ImageDigest = axImageDigest
		}
		if in.RunnerVersion == nil {
			in.RunnerVersion = axRunnerVersion
		}
		if in.SandboxClass == nil {
			in.SandboxClass = axSandboxClass
		}
	}
	refusals := selfLint(&in)
	if len(refusals) > 0 {
		for _, r := range refusals {
			fmt.Fprintln(os.Stderr, "REFUSED "+r)
		}
		fmt.Fprintf(os.Stderr, "emit refused: %d honesty-gate violation(s); output %s NOT written\n", len(refusals), *outPath)
		os.Exit(1)
	}
	out, err := emit(&in)
	if err != nil {
		fmt.Fprintf(os.Stderr, "REFUSED: emit failed: %v\n", err)
		os.Exit(2)
	}
	if err := os.WriteFile(*outPath, out, 0o644); err != nil {
		fmt.Fprintf(os.Stderr, "REFUSED: cannot write output: %v\n", err)
		os.Exit(2)
	}
	fmt.Printf("EMITTED %s (%d bytes) transcript-root sha256 sealed\n", *outPath, len(out))
	os.Exit(0)
}
