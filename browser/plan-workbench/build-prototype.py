"""Build pinned native/WASM, independent CLI controls and project-prefixed assets."""
import argparse,json,os,shutil,subprocess
from pathlib import Path
from controls import requests
p=argparse.ArgumentParser();p.add_argument('--bindgen',required=True,type=Path);p.add_argument('--native-profile',choices=['dev','rplan-replay'],default='dev');a=p.parse_args()
B=Path(__file__).resolve().parent;R=B.parents[1];C=B/'.cache';C.mkdir(exist_ok=True)
env=os.environ.copy();env.pop('GIT_COMMIT',None);target=Path(env.get('CARGO_TARGET_DIR',R/'target/cargo')).resolve();env['CARGO_TARGET_DIR']=str(target);env['CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS']='-C link-arg=--max-memory=67108864'
def run(argv):subprocess.run(list(map(str,argv)),cwd=R,env=env,check=True)
assert subprocess.check_output([str(a.bindgen.resolve()),'--version']).decode().strip()=='wasm-bindgen 0.2.127'
run(['python',B/'verify-foundation.py'])
profile=[] if a.native_profile=='dev' else ['--config','profile.rplan-replay.inherits="dev"','--profile','rplan-replay']
run(['cargo','+1.95.0','build','-p','rplan-cli','--locked',*profile])
for extra in [[],['--lib','--target','wasm32-unknown-unknown']]:run(['cargo','+1.95.0','build','--manifest-path',B/'Cargo.toml','--release','--locked',*extra])
suffix='.exe' if os.name=='nt' else '';original=target/('debug' if a.native_profile=='dev' else a.native_profile)/('rplan'+suffix)
run(['python',B/'capture-cli-controls.py','--binary',original,'--output',C/'cli-current.json'])
run(['python',B/'qualify-native.py','--binary',target/'release'/('rplan-workbench-control'+suffix),'--original',original])
site=R/'target/plan-prototype/RPLAN';site.mkdir(parents=True,exist_ok=True);shutil.copytree(B/'web',site,dirs_exist_ok=True)
for name,example in [('example.json',requests(R)[0]),('disconnected.json',requests(R)[1])]:
    (site/name).write_text(json.dumps(example,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run([a.bindgen.resolve(),target/'wasm32-unknown-unknown/release/rplan_plan_workbench.wasm','--target','web','--out-dir',site/'pkg'])
for path in (site/'pkg').glob('*.d.ts'):path.unlink()
print(json.dumps({'site':str(site),'wasm_bytes':(site/'pkg/rplan_plan_workbench_bg.wasm').stat().st_size}))
