use pulldown_cmark::TagEnd;

fn main() {
    let t = TagEnd::Item;
    match t {
        TagEnd::Paragraph => println!("P"),
        TagEnd::Heading(_) => println!("H"),
        TagEnd::BlockQuote => println!("BQ"),
        TagEnd::CodeBlock => println!("CB"),
        TagEnd::List(_) => println!("L"),
        TagEnd::Item => println!("I"),
        TagEnd::Emphasis => println!("Em"),
        TagEnd::Strong => println!("St"),
        TagEnd::Strikethrough => println!("Str"),
        TagEnd::Link => println!("Ln"),
        TagEnd::Image => println!("Im"),
        _ => println!("Other"),
    }
}
