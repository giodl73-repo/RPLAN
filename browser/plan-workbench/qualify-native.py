"""Repeat complete native responses and compare original CLI audit/verify semantics."""
import argparse,copy,hashlib,json,subprocess,tempfile
from pathlib import Path
from controls import requests
p=argparse.ArgumentParser();p.add_argument('--binary',required=True,type=Path);p.add_argument('--original',required=True,type=Path);args=p.parse_args()
B=Path(__file__).resolve().parent;R=B.parents[1];C=B/'.cache';C.mkdir(exist_ok=True)
records=[];controls=[]
with tempfile.TemporaryDirectory(prefix='rplan independent cli ') as tmp:
    tmp=Path(tmp)
    def cli(argv):return subprocess.run([str(args.original.resolve()),*map(str,argv)],cwd=R,capture_output=True)
    for index,request in enumerate(requests(R)):
        raw=request if isinstance(request,str) else json.dumps(request,ensure_ascii=False,separators=(',',':'))
        def run():return subprocess.run([str(args.binary.resolve())],input=raw.encode(),capture_output=True,check=True).stdout
        output=run();assert output==run(),index;response=json.loads(output)
        records.append({'request':raw,'response_text':output.decode(),'response':response})
        if isinstance(request,str):continue
        paths={}
        for field in ['plan','context','profile','certificate']:
            if request.get(field) is not None:
                path=tmp/(field+'.json');path.write_bytes(request[field].encode());paths[field]=path
        if request['mode']=='audit' and response['kind']=='report':
            argv=['audit','--plan',paths['plan'],'--legal-profile',paths['profile'],'--constraints',','.join(request['constraints']),'--fixed-generated-at',request['generated_at']]
            if 'context' in paths:argv+=['--context',paths['context']]
            if request.get('lineage'):
                lineage=json.loads(request['lineage']);argv+=['--lineage-producer-crate',lineage['producer_crate'],'--lineage-producer-version',lineage['producer_version'],'--lineage-method',lineage['method'],'--lineage-extra-json',json.dumps(lineage['extra'])]
                for parent in lineage['parent_plan_hashes']:argv+=['--lineage-parent-plan-hash',parent]
            result=cli(argv);assert result.returncode in [0,1],result.stderr;native=json.loads(result.stdout);expected=response['certificate']
            omitted={'certificate_id','content_hash','runtime'}
            assert {k:v for k,v in native.items() if k not in omitted}=={k:v for k,v in expected.items() if k not in omitted},index
            a=copy.deepcopy(native['runtime']);b=copy.deepcopy(expected['runtime']);assert a.pop('binary_name')=='rplan';assert b.pop('binary_name')=='rplan-plan-workbench';assert a==b
            assert result.returncode==(0 if expected['result']=='pass' else 1)
            certificate=tmp/'produced-certificate.json';certificate.write_bytes(response['certificate_json'].encode())
            verify=['verify-certificate','--certificate',certificate,'--plan',paths['plan']]
            if 'context' in paths:verify+=['--context',paths['context']]
            verified=cli(verify);assert verified.returncode==0,(index,verified.stderr);identity=json.loads(verified.stdout);assert identity['content_hash']==expected['content_hash'] and identity['result']==expected['result']
            controls.append({'request_index':index,'mode':'audit-and-produced-certificate-verification','exit_code':result.returncode,'all_certificate_semantics_except_runtime_derived_hashes_identical':True,'companion_certificate_verifies_with_original_cli':True})
        elif request['mode']=='verify':
            argv=['verify-certificate','--certificate',paths['certificate']]
            for field in ['plan','context']:
                if field in paths:argv+=['--'+field,paths[field]]
            result=cli(argv);assert (result.returncode==0)==(response['kind']=='report'),index
            if result.returncode==0:
                verified=json.loads(result.stdout);assert verified.pop('verification')=='pass';assert verified==response['verification'],index
            controls.append({'request_index':index,'mode':'verify','exit_code':result.returncode,'outcome_and_successful_identity_fields_identical':True})
for name,data in [('native-cases.json',records),('cli-controls.json',controls)]:
    (C/name).write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
proof={'native_cases':len(records),'each_native_repeated':2,'independent_cli_controls':len(controls),'cli_comparison':'Full audit fields except explicit companion runtime label and derived hashes; every produced companion certificate independently verifies with original CLI. Existing certificate identity fields and error outcomes match.'}
(C/'native-controls.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8');print(json.dumps(proof))
