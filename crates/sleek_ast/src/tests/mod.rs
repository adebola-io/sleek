#[cfg(test)]
mod tests {
    use sleek_utils::Node;

    use crate::{Element, ElementRef, HtmlTag as Tag, Query};

    #[test]
    fn it_parses_class_selector() {
        let mut aside = ElementRef::new("aside");
        aside.add_class("sidebar");
        aside.add_class("scrollable");
        aside.add_class("blue-bg");
        assert!(aside.matches(".sidebar"));
        assert!(aside.matches(".sidebar.scrollable"));
        assert!(aside.matches(".sidebar.blue-bg.scrollable"))
    }

    #[test]
    fn it_parses_id_and_class_selector() {
        let mut para = ElementRef::new("p");
        para.set_attribute("id", "paragraph-1");
        para.add_class("paragraph");
        assert!(para.matches("#paragraph-1.paragraph"));
        assert!(para.matches(".paragraph#paragraph-1"))
    }
    #[test]
    fn it_parses_tag_selector() {
        let mut div = ElementRef::new("div");
        div.add_class("box");
        div.set_attribute("id", "box-1");

        assert!(div.matches("div"));
        assert!(div.matches("div.box"));
        assert!(div.matches("div#box-1"));
        assert!(div.matches("div.box#box-1"));
    }

    #[test]
    fn it_parses_attributes() {
        let mut button = ElementRef::new("button");
        button.add_class("clickable");
        button.set_attribute("title", "Click Me");
        button.set_attribute("style", "background-color: blue;");
        button.set_attribute("darkmode", "");

        assert!(button.matches("button.clickable[darkmode]"));
        assert!(button.matches("[style]"));
        // assert!(button.matches("[title=\"Click Me\"]"))
    }

    #[test]
    fn it_parses_descendants() {
        let mut div = ElementRef::new("div");
        div.add_class("container");

        let mut button = ElementRef::new("button");
        button.set_attribute("id", "button-1");
        button.add_class("bg-transparent");

        let mut span = ElementRef::new("span");
        span.add_class("text-red-500");
        span.set_attribute("title", "Click Me!");

        div.append(&button);
        button.append(&span);

        assert!(span.matches("div button span"));
        assert!(span.matches(".container .bg-transparent .text-red-500"));
        assert!(span.matches("div [title]"));
        assert!(span.matches("#button-1 .text-red-500[title]"));
    }

    #[test]
    fn it_parses_nuclear_relations() {
        let mut html = ElementRef::new("html");
        let mut head = ElementRef::new("head");
        let title = ElementRef::new("title");

        html.append(&head);
        head.append(&title);

        assert!(title.matches("head > title"));
        assert!(title.matches("html > head > title"));
    }

    #[test]
    fn it_tests_universal_selector() {
        let mut ul = ElementRef::new("ul");
        ul.add_class("list");

        let mut item = ElementRef::new("list");
        ul.append(&item);
        item.add_class("list-item");

        let p = ElementRef::new("p");
        item.append(&p);

        assert!(p.matches("*"));
        assert!(p.matches("* *"));
        assert!(p.matches("* * *"));
        assert!(p.matches("* p"));
        assert!(p.matches(".list  *"));
        assert!(p.matches("* > p"));
        assert!(p.matches("* > [class] > p"));
    }

    #[test]
    fn it_tests_element_nesting() {
        let ref_1 = ElementRef::from(Tag::Div);
        let mut ref_2 = ElementRef::from(Tag::Body);
        ref_2.append(&ref_1);

        let mut ref_3 = ElementRef::from(Tag::Html);
        ref_3.append(&ref_2);

        let ref_4 = ElementRef::from(Tag::Head);

        assert!(ref_3.contains(&ref_1));
        ref_3.append(&ref_4);

        assert_eq!(ref_3.children().len(), 2);
    }

    #[test]
    fn it_updates_class_list() {
        let mut ref_1 = ElementRef::from(Tag::Div);
        ref_1.set_attribute("class", "box blue");
        assert_eq!(ref_1.class_list(), ["box", "blue"])
    }

    #[test]
    fn it_queries_matching() {
        let mut div = ElementRef::from(Tag::Div);
        let mut child_2 = ElementRef::from(Tag::Div);
        let mut child_3 = ElementRef::from(Tag::A);

        div.append(&child_2);
        div.append(&child_3);

        child_3.set_attribute("class", "box");
        child_2.set_attribute("id", "inner-circle");

        assert_eq!(div.get_elements_by_class_name("box")[0], child_3);
        assert_eq!(div.get_element_by_id("inner-circle"), Some(child_2));
        assert_eq!(div.get_elements_by_tag_name(&Tag::A)[0], child_3);
    }

    #[test]
    fn it_adds_element_after() {
        let mut div = ElementRef::from(Tag::Div);
        let mut child_1 = ElementRef::from(Tag::Div);
        let child_2 = ElementRef::from(Tag::Div);

        div.append(&child_1);
        child_1.after(&child_2);

        assert_eq!(div.children().first().unwrap(), &child_1);

        assert_eq!(div.children().len(), 2);
    }

    #[test]
    fn it_tests_element_removal() {
        let node = ElementRef::from(Tag::Div);
        let mut body = ElementRef::from(Tag::Body);

        body.append(&node);

        body.remove(&node);

        assert_eq!(node.parent(), None);
        assert_eq!(body.children().len(), 0);
    }

    #[test]
    fn it_tests_element_ref_equality() {
        let element = Element::new(Tag::Div);
        let ref_1 = ElementRef::over(element);
        let ref_2 = ref_1.clone();
        assert_eq!(ref_1, ref_2);
    }
}
