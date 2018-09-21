extern crate ignore;

use ignore::WalkBuilder;

fn main() {
  let walk = WalkBuilder::new("./")
    .follow_links(false)
    .standard_filters(true)
    .same_file_system(true)
    .build();

  for entry in walk {
    println!("{:?}", entry);
  }
}
