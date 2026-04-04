import "common" as common;

const IS_ALL = true;

fn admit_event(event) {
  if event.get("tags")?.some(|tag| !tag.is_empty() && tag[0] == "t") {
    throw "Event can't contains hashtags";
  }
}

fn admit_query(filter) {
  if common::not_unit_and(filter.get("#t"), false, |hashtags| !hashtags.is_empty()) {
    throw "This relay doesn't support hashtags for some reason";
  }
}
