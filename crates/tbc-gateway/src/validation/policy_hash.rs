pub fn verify_policy_hash(computed: &str, provided: &str) -> bool {
    computed == provided
}