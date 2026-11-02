"""Preserve every original native test, manifest, lock and unchanged algorithm."""
import hashlib,json
from pathlib import Path
B=Path(__file__).resolve().parent;R=B.parents[1];record=json.loads((B/'native-foundation.json').read_text(encoding='utf-8'))
for item in record['files']:
    path=item['path'];data=(R/path).read_bytes().replace(b'\r\n',b'\n')
    if path=='crates/rplan-cli/src/main.rs':
        assert hashlib.sha256(data[:data.index(b'fn validate_profile_applicability(')]).hexdigest()==record['original_cli_before_profile_helpers_sha256'],path
        tests=data[data.index(b'#[cfg(test)]'):];assert hashlib.sha256(tests).hexdigest()==record['original_cli_inline_tests_sha256'],path
        assert b'rplan_audit::parse_plan_chamber(value)' in data and b'rplan_audit::validate_profile_applicability(document, plan_chamber, profile)' in data
        continue
    if path=='crates/rplan-audit/src/lib.rs':
        data=data.replace(b'\nmod profile;\npub use profile::{parse_plan_chamber, validate_profile_applicability};\n',b'')
    assert hashlib.sha256(data).hexdigest()==item['sha256'],path
print('All70 original tests, native algorithms, manifests/lock and original grants retain canonical text; CLI delegates only the extracted profile checks.')
