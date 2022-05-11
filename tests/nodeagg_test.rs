use nodeagg::Nodeagg;

#[test]
fn test_try_from() {
    let hostnames: Vec<String> = Nodeagg::try_from("node[01-02],node03").unwrap().iter().collect();
    assert_eq!(
        hostnames, vec!["node01", "node02", "node03"]);
}
