"""Canonical owned fixtures plus explicit caller-byte, policy and admission cases."""
import copy,json

def requests(root):
    def text(path):return (root/path).read_text(encoding='utf-8')
    folder='crates/rplan-audit/fixtures/'
    base={'mode':'audit','plan':text(folder+'grid3x3-valid.rplan'),'context':text(folder+'grid3x3.rctx'),'profile':text('crates/rplan-audit/profiles/us-congressional-project-v1.json'),'constraints':['plan-shape','population','contiguity'],'generated_at':'2026-05-10T00:00:00Z'}
    cases=[copy.deepcopy(base)]
    def changed(**fields):cases.append(dict(copy.deepcopy(base),**fields))
    changed(plan=text(folder+'grid3x3-disconnected.rplan'))
    changed(context=None)
    changed(constraints=['plan-shape'])
    changed(constraints=['plan-shape','population','contiguity','splits','vra','geometry'])
    changed(constraints=['plan-shape','vra'])
    changed(constraints=['population','population'])
    for field,value in [('year',2021),('jurisdiction','XX'),('chamber','state-house')]:
        profile=json.loads(base['profile']);profile[field]=value;changed(profile=json.dumps(profile))
    plan=json.loads(base['plan']);plan['metadata']['chamber']='unknown';changed(plan=json.dumps(plan))
    plan=json.loads(base['plan']);plan['plan']['k']=18446744073709551615;changed(plan=json.dumps(plan))
    plan=json.loads(base['plan']);plan['plan']['assignment'][0]=123;changed(plan=json.dumps(plan))
    plan=json.loads(base['plan']);plan['unrecognized']='<script>untrusted💡</script>';plan['extensions']['caller']={'status':'approved','counter':18446744073709551615};changed(plan=json.dumps(plan,ensure_ascii=False))
    plan=json.loads(base['plan']);plan['plan']['schema_version']='caller-schema';changed(plan=json.dumps(plan))
    context=json.loads(base['context']);context['context_hash']='sha256:stale';changed(context=json.dumps(context))
    context=json.loads(base['context']);context['populations']=[9223372036854775807]*len(context['units']['unit_ids']);changed(context=json.dumps(context))
    changed(profile=None)
    changed(plan='')
    changed(plan='{')
    changed(plan='\ufeff'+base['plan'])
    changed(plan=base['plan'].replace('\n','\r\n'))
    changed(plan=' '.ljust(262145))
    changed(generated_at='💡'*33)
    changed(lineage=json.dumps({'producer_crate':'caller','producer_version':'1','method':'synthetic','parent_plan_hashes':[],'extra':{'counter':18446744073709551615}}))
    changed(lineage=json.dumps({'producer_crate':'caller','producer_version':'1','method':'synthetic','parent_plan_hashes':[],'extra':{'result':'pass'}}))
    changed(unexpected=True)
    legacy={'rplan_version':'0.1','metadata':{'label':'legacy','state_code':'WA','year':'2020','chamber':'congressional','num_districts':2,'created_at':'2026-05-10'},'assignments':{'53001000100':1,'53001000200':2},'geometry':None}
    changed(plan=json.dumps(legacy),context=None,constraints=['plan-shape'])
    for cert in ['grid3x3-valid-certificate.json','grid3x3-disconnected-certificate.json','grid3x3-missing-contiguity-certificate.json']:
        cases.append({'mode':'verify','certificate':text(folder+cert),'plan':base['plan'],'context':base['context']})
    cases.append({'mode':'verify','certificate':text(folder+'grid3x3-valid-certificate.json')})
    certificate=json.loads(text(folder+'grid3x3-valid-certificate.json'));certificate['result']='fail'
    cases.append({'mode':'verify','certificate':json.dumps(certificate),'plan':base['plan'],'context':base['context']})
    cases+=['{',' '*1048577]
    return cases
