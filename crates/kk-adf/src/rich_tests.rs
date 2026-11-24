use crate::{from_storage, to_storage, Doc};
use serde_json::{json, Value};

fn adf(xml: &str) -> Value {
    serde_json::to_value(from_storage(xml).unwrap()).unwrap()["content"].clone()
}

fn doc(content: Value) -> Doc {
    serde_json::from_value(json!({"version": 1, "type": "doc", "content": content})).unwrap()
}

/// ADF -> storage must equal `xml`, and storage -> ADF must return the same ADF.
fn both(content: Value, xml: &str) {
    let d = doc(content);
    assert_eq!(to_storage(&d), xml);
    assert_eq!(from_storage(xml).unwrap(), d);
}

fn p(inline: Value) -> Value {
    json!({"type": "paragraph", "content": inline})
}

fn t(s: &str) -> Value {
    json!({"type": "text", "text": s})
}

#[test]
fn panels_map_to_named_macros() {
    let body = "<ac:rich-text-body><p>hi</p></ac:rich-text-body></ac:structured-macro>";
    for (ty, name) in [
        ("info", "info"),
        ("note", "note"),
        ("warning", "warning"),
        ("success", "tip"),
    ] {
        both(
            json!([{"type": "panel", "attrs": {"panelType": ty}, "content": [p(json!([t("hi")]))]}]),
            &format!("<ac:structured-macro ac:name=\"{name}\" ac:schema-version=\"1\">{body}"),
        );
    }
    both(
        json!([{"type": "panel", "attrs": {"panelType": "error", "localId": "m1"}, "content": [p(json!([t("hi")]))]}]),
        &format!("<ac:structured-macro ac:name=\"panel\" ac:schema-version=\"1\" ac:macro-id=\"m1\"><ac:parameter ac:name=\"panelType\">error</ac:parameter>{body}"),
    );
}

#[test]
fn custom_panel_stays_lossless_through_fallback() {
    let d = doc(
        json!([{"type": "panel", "attrs": {"panelType": "custom", "panelColor": "#fff"},
        "content": [p(json!([t("x")]))]}]),
    );
    assert!(to_storage(&d).starts_with("<kk:adf-node>"));
    assert_eq!(from_storage(&to_storage(&d)).unwrap(), d);
}

#[test]
fn expand_and_nested_expand() {
    let inner = json!([p(json!([t("a")]))]);
    both(
        json!([{"type": "expand", "attrs": {"title": "More"}, "content": inner}]),
        "<ac:structured-macro ac:name=\"expand\" ac:schema-version=\"1\"><ac:parameter ac:name=\"title\">More</ac:parameter><ac:rich-text-body><p>a</p></ac:rich-text-body></ac:structured-macro>",
    );
    let cell = json!([{"type": "table", "content": [{"type": "tableRow", "content": [{"type": "tableCell",
        "content": [{"type": "nestedExpand", "attrs": {"title": "In"}, "content": inner}]}]}]}]);
    both(
        cell,
        "<table><tbody><tr><td><ac:structured-macro ac:name=\"expand\" ac:schema-version=\"1\"><ac:parameter ac:name=\"title\">In</ac:parameter><ac:rich-text-body><p>a</p></ac:rich-text-body></ac:structured-macro></td></tr></tbody></table>",
    );
}

#[test]
fn inline_status_date_mention_emoji_card() {
    let inline = json!([
        {"type": "status", "attrs": {"text": "Done", "color": "green", "localId": "s1"}},
        t(" "),
        {"type": "date", "attrs": {"timestamp": "1699920000000"}},
        t(" "),
        {"type": "mention", "attrs": {"id": "42", "text": "@Ann"}},
        t(" "),
        {"type": "emoji", "attrs": {"shortName": ":smile:", "id": "1f604", "text": "x"}},
        t(" "),
        {"type": "inlineCard", "attrs": {"url": "https://e.io/a?b=1&c=2"}},
    ]);
    both(
        json!([p(inline)]),
        "<p><ac:structured-macro ac:name=\"status\" ac:schema-version=\"1\" ac:macro-id=\"s1\">\
         <ac:parameter ac:name=\"title\">Done</ac:parameter><ac:parameter ac:name=\"colour\">Green</ac:parameter>\
         </ac:structured-macro> <time datetime=\"2023-11-14\" /> \
         <ac:link><ri:user ri:account-id=\"42\"/><ac:plain-text-link-body><![CDATA[@Ann]]></ac:plain-text-link-body></ac:link> \
         <ac:emoticon ac:name=\"smile\" ac:emoji-shortname=\":smile:\" ac:emoji-id=\"1f604\" ac:emoji-fallback=\"x\"/> \
         <a href=\"https://e.io/a?b=1&amp;c=2\" data-card-appearance=\"inline\">https://e.io/a?b=1&amp;c=2</a></p>",
    );
}

#[test]
fn reads_reference_shaped_inline_forms() {
    assert_eq!(
        adf("<p><ac:emoticon ac:name=\"smile\"/><ac:structured-macro ac:name=\"status\"><ac:parameter ac:name=\"colour\">Grey</ac:parameter></ac:structured-macro></p>")[0]["content"],
        json!([
            {"type": "emoji", "attrs": {"shortName": ":smile:"}},
            {"type": "status", "attrs": {"text": "", "color": "neutral"}},
        ])
    );
    assert_eq!(
        adf("<p><ac:link><ri:user ri:account-id=\"7\"/></ac:link></p>")[0]["content"],
        json!([{"type": "mention", "attrs": {"id": "7"}}])
    );
}

#[test]
fn bad_dates_and_unmappable_inlines_stay_verbatim() {
    let v = adf("<p><time datetime=\"2023-02-30\" /></p>");
    assert_eq!(v[0]["content"][0]["type"], "inlineExtension");
    assert_eq!(v[0]["content"][0]["attrs"]["extensionType"], "kk.storage");
    let d = doc(json!([p(
        json!([{"type": "date", "attrs": {"timestamp": "1699920000001"}}])
    )]));
    assert_eq!(from_storage(&to_storage(&d)).unwrap(), d);
}

#[test]
fn tasks_and_decisions() {
    both(
        json!([{"type": "taskList", "attrs": {"localId": "l1"}, "content": [
            {"type": "taskItem", "attrs": {"localId": "u1", "state": "TODO"}, "content": [t("a")]},
            {"type": "taskItem", "attrs": {"localId": "u2", "state": "DONE"}, "content": [t("b")]},
        ]}]),
        "<ac:task-list kk:local-id=\"l1\">\
         <ac:task><ac:task-id>1</ac:task-id><ac:task-uuid>u1</ac:task-uuid><ac:task-status>incomplete</ac:task-status><ac:task-body>a</ac:task-body></ac:task>\
         <ac:task><ac:task-id>2</ac:task-id><ac:task-uuid>u2</ac:task-uuid><ac:task-status>complete</ac:task-status><ac:task-body>b</ac:task-body></ac:task>\
         </ac:task-list>",
    );
    both(
        json!([{"type": "decisionList", "attrs": {"localId": "d1"}, "content": [
            {"type": "decisionItem", "attrs": {"localId": "d2", "state": "DECIDED"}, "content": [t("go")]},
        ]}]),
        "<ac:adf-extension><ac:adf-node type=\"decision-list\"><ac:adf-attribute key=\"local-id\">d1</ac:adf-attribute>\
         <ac:adf-node type=\"decision-item\"><ac:adf-attribute key=\"local-id\">d2</ac:adf-attribute>\
         <ac:adf-attribute key=\"state\">DECIDED</ac:adf-attribute><ac:adf-content>go</ac:adf-content></ac:adf-node>\
         </ac:adf-node></ac:adf-extension>",
    );
}

#[test]
fn media_single_group_and_external() {
    both(
        json!([{"type": "mediaSingle", "attrs": {"layout": "center", "width": 400}, "content": [
            {"type": "media", "attrs": {"type": "file", "id": "f-1", "collection": "c9", "__fileName": "a.png", "alt": "A", "width": 800, "height": 600}},
            {"type": "caption", "content": [t("cap")]},
        ]}]),
        "<p><ac:image ac:layout=\"center\" ac:width=\"400\" ac:alt=\"A\" ac:original-width=\"800\" ac:original-height=\"600\">\
         <ri:attachment ri:filename=\"a.png\" kk:media-id=\"f-1\" kk:collection=\"c9\"/><ac:caption>cap</ac:caption></ac:image></p>",
    );
    both(
        json!([{"type": "mediaSingle", "content": [{"type": "media", "attrs": {"type": "external", "url": "https://e.io/x.png"}}]}]),
        "<p><ac:image><ri:url ri:value=\"https://e.io/x.png\"/></ac:image></p>",
    );
    both(
        json!([{"type": "mediaGroup", "content": [
            {"type": "media", "attrs": {"type": "file", "__fileName": "a.pdf"}},
            {"type": "media", "attrs": {"type": "file", "__fileName": "b.pdf"}},
        ]}]),
        "<ac:structured-macro ac:name=\"view-file\" ac:schema-version=\"1\">\
         <ac:parameter ac:name=\"name\"><ri:attachment ri:filename=\"a.pdf\"/></ac:parameter>\
         <ac:parameter ac:name=\"name\"><ri:attachment ri:filename=\"b.pdf\"/></ac:parameter></ac:structured-macro>",
    );
}

#[test]
fn layouts_and_block_card() {
    both(
        json!([{"type": "layoutSection", "content": [
            {"type": "layoutColumn", "attrs": {"width": 66.66}, "content": [p(json!([t("l")]))]},
            {"type": "layoutColumn", "attrs": {"width": 33.33}, "content": [p(json!([t("r")]))]},
        ]}]),
        "<ac:layout><ac:layout-section ac:type=\"two_left_sidebar\">\
         <ac:layout-cell><p>l</p></ac:layout-cell><ac:layout-cell><p>r</p></ac:layout-cell></ac:layout-section></ac:layout>",
    );
    let two = adf("<ac:layout><ac:layout-section ac:type=\"single\"><ac:layout-cell><p>a</p></ac:layout-cell></ac:layout-section>\
                   <ac:layout-section ac:type=\"three_with_sidebars\"><ac:layout-cell/><ac:layout-cell/><ac:layout-cell/></ac:layout-section></ac:layout>");
    assert_eq!(two.as_array().unwrap().len(), 2);
    assert_eq!(two[1]["content"][1]["attrs"]["width"], json!(50.0));
    both(
        json!([{"type": "blockCard", "attrs": {"url": "https://e.io/p"}}]),
        "<p><a href=\"https://e.io/p\" data-card-appearance=\"block\">https://e.io/p</a></p>",
    );
}

#[test]
fn styled_marks() {
    let m = |k: &str, a: Value| json!({"type": k, "attrs": a});
    both(
        json!([p(json!([
            {"type": "text", "text": "r", "marks": [m("textColor", json!({"color": "#ff0000"})), m("backgroundColor", json!({"color": "#00ff00"}))]},
            {"type": "text", "text": "2", "marks": [m("subsup", json!({"type": "sup"}))]},
            {"type": "text", "text": "c", "marks": [m("annotation", json!({"id": "abc", "annotationType": "inlineComment"}))]},
            {"type": "text", "text": "d", "marks": [{"type": "strike"}, {"type": "underline"}]},
        ]))]),
        "<p><span style=\"color: #ff0000;\"><span style=\"background-color: #00ff00;\">r</span></span>\
         <sup>2</sup><ac:inline-comment-marker ac:ref=\"abc\">c</ac:inline-comment-marker><s><u>d</u></s></p>",
    );
    both(
        json!([
            {"type": "paragraph", "marks": [m("alignment", json!({"align": "center"})), m("indentation", json!({"level": 2}))], "content": [t("x")]},
            {"type": "heading", "attrs": {"level": 2}, "marks": [m("alignment", json!({"align": "end"}))], "content": [t("h")]},
        ]),
        "<p style=\"text-align: center; margin-left: 60px;\">x</p><h2 style=\"text-align: right;\">h</h2>",
    );
}

#[test]
fn unknown_marks_and_attrs_fall_back_without_loss() {
    let d = doc(json!([
        {"type": "paragraph", "marks": [{"type": "breakout", "attrs": {"mode": "wide"}}], "content": [t("x")]},
        p(json!([{"type": "text", "text": "l", "marks": [{"type": "link", "attrs": {"href": "u", "title": "t"}}]}])),
    ]));
    let xml = to_storage(&d);
    assert_eq!(xml.matches("<kk:adf-node>").count(), 2, "{xml}");
    assert_eq!(from_storage(&xml).unwrap(), d);
}

#[test]
fn macros_become_extensions() {
    both(
        json!([
            {"type": "extension", "attrs": {"extensionType": "kk.macro", "extensionKey": "toc", "localId": "m9",
                "parameters": {"macroParams": {"maxLevel": {"value": "3"}}, "macroMetadata": {"schemaVersion": {"value": "1"}}}}},
            {"type": "bodiedExtension", "attrs": {"extensionType": "kk.macro", "extensionKey": "details", "parameters": {}},
                "content": [p(json!([t("k")]))]},
            {"type": "extension", "attrs": {"extensionType": "kk.macro", "extensionKey": "noformat", "parameters": {}, "text": "a]]>b"}},
        ]),
        "<ac:structured-macro ac:name=\"toc\" ac:schema-version=\"1\" ac:macro-id=\"m9\"><ac:parameter ac:name=\"maxLevel\">3</ac:parameter></ac:structured-macro>\
         <ac:structured-macro ac:name=\"details\"><ac:rich-text-body><p>k</p></ac:rich-text-body></ac:structured-macro>\
         <ac:structured-macro ac:name=\"noformat\"><ac:plain-text-body><![CDATA[a]]]]><![CDATA[>b]]></ac:plain-text-body></ac:structured-macro>",
    );
    assert_eq!(
        adf("<p>x <ac:structured-macro ac:name=\"anchor\"><ac:parameter ac:name=\"\">top</ac:parameter></ac:structured-macro></p>")[0]["content"][1]["type"],
        "inlineExtension"
    );
}

#[test]
fn code_macro_keeps_macro_id_and_extra_params_become_extension() {
    both(
        json!([{"type": "codeBlock", "attrs": {"language": "rust", "uniqueId": "c1"}, "content": [t("fn x() {}")]}]),
        "<ac:structured-macro ac:name=\"code\" ac:schema-version=\"1\" ac:macro-id=\"c1\"><ac:parameter ac:name=\"language\">rust</ac:parameter>\
         <ac:plain-text-body><![CDATA[fn x() {}]]></ac:plain-text-body></ac:structured-macro>",
    );
    let v = adf("<ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"title\">T</ac:parameter><ac:plain-text-body><![CDATA[x]]></ac:plain-text-body></ac:structured-macro>");
    assert_eq!(
        v[0]["attrs"],
        json!({"extensionType": "kk.macro", "extensionKey": "code", "text": "x",
               "parameters": {"macroParams": {"title": {"value": "T"}}}})
    );
}
