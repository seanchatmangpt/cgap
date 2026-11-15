import subprocess,sys,re,glob,os,hashlib
import rdflib
P="/home/user/cgap/docs/jira/v26.11.14/c1-docs-diataxis-pack"
fx=open(P+"/qualification/consumer.ttl").read()
def sub(a,b,c=1):
    assert a in fx,a; return fx.replace(a,b,c)
M={
"BASE":(fx,None),
"M1 strip task":(sub(' ; ddx:task "rotate keys"',''),"010"),
"M2 two quadrants":(sub('ddx:quadrant ddx:Tutorial ; ddx:position 1','ddx:quadrant ddx:Tutorial, ddx:HowTo ; ddx:position 7 ; ddx:task "t"'),"020"),  # also trips nav 030 (n1 navOrder stale) - correct
"M3 dup position":(sub('ddx:position 2','ddx:position 1'),"030"),
"M4 dup relPath":(sub('tutorials/second.md','tutorials/start.md'),"040"),
"M5 quadrant Blog":(sub('ddx:quadrant ddx:HowTo','ddx:quadrant ddx:Blog'),"020"),
"M6 .txt":(sub('how-to/rotate.md','how-to/rotate.txt'),"040"),
"M7 empty graph":("@prefix ddx: <https://seanchatmangpt.github.io/ggen-marketplace/docs-diataxis#> .\n","050"),
"M8 nav mismatch":(sub('ddx:navOrder 1002','ddx:navOrder 9999'),"030"),
"M9 two docs":(sub('ex:doc a ddx:Doc','ex:doc2 a ddx:Doc ; dcterms:title "x" .\nex:doc a ddx:Doc'),"050"),
"M10 strip quadrant":(sub('ddx:quadrant ddx:Reference ; ',''),"010"),
}
ok=True
for k,(t,exp) in M.items():
    f=os.environ.get('S','/tmp')+'/m.ttl'; open(f,"w").write(t)
    r=subprocess.run(["python3",P+"/reference_gate.py",f],capture_output=True,text=True)
    pg=set(re.findall(r"gate (\d+)",r.stderr))
    g=rdflib.Graph(); g.parse(data=t,format="turtle")
    o=rdflib.Graph(); o.parse(P+"/ontology.ttl")
    g+=o
    sg=set()
    for q in sorted(glob.glob(P+"/gates/*.rq")):
        n=os.path.basename(q)[:3]
        if len(list(g.query(open(q).read())))>0: sg.add(n)
    good=(pg==sg and exp in (pg|sg)) if k.startswith('M2') else (pg==({exp} if exp else set()) or (exp=="050" and r.returncode==2 and not pg)) and sg==({exp} if exp else set())
    ok&=good
    print(f"{k}: ref rc={r.returncode} gates={sorted(pg)} sparql={sorted(sg)} expect={exp} {'OK' if good else 'FAIL'}")
sys.exit(0 if ok else 1)
