const IS_ALL = true;

fn admit_query(filter) {
  if filter.get("kinds")?.contains(1) {
    throw "Exploring relay notes is not permitted";
  }
}
