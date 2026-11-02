use std::io::{Read, Write};

fn main() {
    let mut bytes = Vec::new();
    let response = match std::io::stdin()
        .take((rplan_plan_workbench::MAX_REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
    {
        Err(_) => rplan_plan_workbench::refusal("Could not read the whole request"),
        Ok(_) if bytes.len() > rplan_plan_workbench::MAX_REQUEST_BYTES => {
            rplan_plan_workbench::refusal("Request exceeds 1 MiB")
        }
        Ok(_) => match std::str::from_utf8(&bytes) {
            Ok(input) => rplan_plan_workbench::evaluate(input),
            Err(_) => rplan_plan_workbench::refusal("Request is not UTF-8"),
        },
    };
    std::io::stdout().write_all(response.as_bytes()).unwrap();
}
