//! The 1.6 action names and policy, compiled in. Entries for later lanes are
//! declarations, not callable handlers. Do not dispatch arbitrary Tauri commands
//! from `commands`: use a typed action match after C5's policy and field checks.

use super::{ActionSpec, ActionUi, Confirm, Level, Undo};

macro_rules! action {
    ($name:literal, $level:ident, $confirm:ident, $undo:ident, $params:literal, $label:literal, $icon:literal, [$($command:literal),*]) => {
        ActionSpec { name: $name, level: Level::$level, confirm: Confirm::$confirm,
            undo: Undo::$undo, params: $params, ui: ActionUi { label: $label, icon: $icon },
            commands: &[$($command),*] }
    };
}

pub static ACTIONS: &[ActionSpec] = &[
    action!(
        "ai.hello",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Ai hello",
        "Bot",
        ["vault_status"]
    ),
    action!(
        "ai.permissions.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Ai permissions get",
        "Bot",
        ["vault_status"]
    ),
    action!(
        "ai.limits.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Ai limits get",
        "Bot",
        ["vault_status"]
    ),
    action!(
        "ui.where",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Ui where",
        "PanelTop",
        []
    ),
    action!(
        "tree.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Tree list",
        "ListTree",
        ["list_notes"]
    ),
    action!(
        "notebook.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Notebook list",
        "BookOpen",
        ["list_notes"]
    ),
    action!(
        "section.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Section list",
        "Folder",
        ["list_notes"]
    ),
    action!(
        "page.recent",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Page recent",
        "FileText",
        ["list_notes"]
    ),
    action!(
        "page.favorites",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Page favorites",
        "FileText",
        ["list_notes"]
    ),
    action!(
        "bin.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Bin list",
        "Trash2",
        ["list_notes"]
    ),
    action!(
        "notebook.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Notebook get",
        "BookOpen",
        ["list_notes"]
    ),
    action!(
        "section.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Section get",
        "Folder",
        ["list_notes"]
    ),
    action!(
        "page.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"},"format":{"enum":["markdown","blocks"]}},"required":["id"],"additionalProperties":false}"#,
        "Page get",
        "FileText",
        ["get_page"]
    ),
    action!(
        "page.outline",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page outline",
        "FileText",
        []
    ),
    action!(
        "block.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Block list",
        "Blocks",
        []
    ),
    action!(
        "db.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Db list",
        "Table2",
        ["list_notes"]
    ),
    action!(
        "db.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Db get",
        "Table2",
        ["get_database"]
    ),
    action!(
        "db.query",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"viewId":{"type":"string"},"offset":{"type":"integer","minimum":0},"limit":{"type":"integer","minimum":1,"maximum":200},"groupOffset":{"type":"integer","minimum":0},"groupLimit":{"type":"integer","minimum":1,"maximum":200},"subGroupOffset":{"type":"integer","minimum":0},"subGroupLimit":{"type":"integer","minimum":1,"maximum":200}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db query",
        "Table2",
        ["query_view", "search_rows"]
    ),
    action!(
        "db.row.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Db row get",
        "Table2",
        ["get_row"]
    ),
    action!(
        "db.row_titles",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Db row titles",
        "Table2",
        ["row_titles"]
    ),
    action!(
        "db.chart.data",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"viewId":{"type":"string"}},"required":["databaseId","viewId"],"additionalProperties":false}"#,
        "Db chart data",
        "Table2",
        ["chart_data"]
    ),
    action!(
        "db.reminders.due",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Db reminders due",
        "Table2",
        ["due_reminders_now"]
    ),
    action!(
        "db.file.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"rowId":{"type":"string"},"propId":{"type":"string"}},"required":["rowId","propId"],"additionalProperties":false}"#,
        "Db file list",
        "Table2",
        []
    ),
    action!(
        "db.formula.test",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"formula":{"type":"string"},"rows":{"type":"array","items":{"type":"object"}}},"required":["formula"],"additionalProperties":false}"#,
        "Db formula test",
        "Table2",
        []
    ),
    action!(
        "sticky.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Sticky list",
        "StickyNote",
        ["list_stickies"]
    ),
    action!(
        "sticky.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Sticky get",
        "StickyNote",
        ["get_sticky"]
    ),
    action!(
        "search.all",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"query":{"type":"string"}},"required":["query"],"additionalProperties":false}"#,
        "Search all",
        "Search",
        ["search"]
    ),
    action!(
        "calc.evaluate",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"expression":{"type":"string"}},"required":["expression"],"additionalProperties":false}"#,
        "Calc evaluate",
        "Calculator",
        []
    ),
    action!(
        "calc.history",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Calc history",
        "Calculator",
        ["calc_history"]
    ),
    action!(
        "settings.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Settings get",
        "Settings",
        ["get_settings"]
    ),
    action!(
        "backup.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Backup list",
        "Archive",
        ["list_backups"]
    ),
    action!(
        "activity.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":[],"additionalProperties":false}"#,
        "Activity list",
        "List",
        []
    ),
    action!(
        "activity.verify",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":[],"additionalProperties":false}"#,
        "Activity verify",
        "List",
        []
    ),
    action!(
        "history.list",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":[],"additionalProperties":false}"#,
        "History list",
        "History",
        []
    ),
    action!(
        "history.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"},"versionId":{"type":"string"}},"required":["id","versionId"],"additionalProperties":false}"#,
        "History get",
        "History",
        []
    ),
    action!(
        "about.get",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "About get",
        "Info",
        []
    ),
    action!(
        "find.by_source",
        Look,
        None,
        None,
        r#"{"type":"object","properties":{"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["source"],"additionalProperties":false}"#,
        "Find by source",
        "Search",
        []
    ),
    action!(
        "ai.claim",
        Edit,
        None,
        None,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Ai claim",
        "Bot",
        []
    ),
    action!(
        "ai.release",
        Edit,
        None,
        None,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Ai release",
        "Bot",
        []
    ),
    action!(
        "ui.open",
        Edit,
        None,
        View,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Ui open",
        "PanelTop",
        []
    ),
    action!(
        "ui.highlight",
        Edit,
        None,
        View,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Ui highlight",
        "PanelTop",
        []
    ),
    action!(
        "ui.notify",
        Edit,
        None,
        None,
        r#"{"type":"object","properties":{"message":{"type":"string"}},"required":["message"],"additionalProperties":false}"#,
        "Ui notify",
        "PanelTop",
        []
    ),
    action!(
        "ui.wait_for_person",
        Edit,
        None,
        None,
        r#"{"type":"object","properties":{"message":{"type":"string"}},"required":["message"],"additionalProperties":false}"#,
        "Ui wait for person",
        "PanelTop",
        []
    ),
    action!(
        "batch.run",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"steps":{"type":"array","items":{"type":"object","properties":{"action":{"type":"string"},"params":{"type":"object"}},"required":["action","params"],"additionalProperties":false}}},"required":["steps"],"additionalProperties":false}"#,
        "Batch run",
        "Layers",
        []
    ),
    action!(
        "notebook.create",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"name":{"type":"string"},"color":{"type":"string"},"icon":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["name"],"additionalProperties":false}"#,
        "Notebook create",
        "BookOpen",
        ["create_notebook"]
    ),
    action!(
        "notebook.update",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"name":{"type":"string"},"color":{"type":"string"},"icon":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id"],"additionalProperties":false}"#,
        "Notebook update",
        "BookOpen",
        ["update_notebook"]
    ),
    action!(
        "section.create",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"notebookId":{"type":"string"},"name":{"type":"string"},"color":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["notebookId","name"],"additionalProperties":false}"#,
        "Section create",
        "Folder",
        ["create_section"]
    ),
    action!(
        "section.update",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"name":{"type":"string"},"color":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id"],"additionalProperties":false}"#,
        "Section update",
        "Folder",
        ["update_section"]
    ),
    action!(
        "notebook.reorder",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Notebook reorder",
        "BookOpen",
        ["reorder"]
    ),
    action!(
        "section.reorder",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Section reorder",
        "Folder",
        ["reorder"]
    ),
    action!(
        "page.reorder",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Page reorder",
        "FileText",
        ["reorder"]
    ),
    action!(
        "db.reorder",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Db reorder",
        "Table2",
        ["reorder"]
    ),
    action!(
        "notebook.trash",
        Edit,
        Ask,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Notebook trash",
        "BookOpen",
        []
    ),
    action!(
        "section.trash",
        Edit,
        Ask,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Section trash",
        "Folder",
        []
    ),
    action!(
        "sticky.trash",
        Edit,
        Ask,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Sticky trash",
        "StickyNote",
        []
    ),
    action!(
        "section.move",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"notebookId":{"type":"string"}},"required":["id","notebookId"],"additionalProperties":false}"#,
        "Section move",
        "Folder",
        []
    ),
    action!(
        "page.create",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"sectionId":{"type":"string"},"title":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["sectionId"],"additionalProperties":false}"#,
        "Page create",
        "FileText",
        ["create_page"]
    ),
    action!(
        "page.create_sub",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"parentId":{"type":"string"},"title":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["parentId"],"additionalProperties":false}"#,
        "Page create sub",
        "FileText",
        ["create_subpage"]
    ),
    action!(
        "page.write",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"format":{"enum":["markdown","blocks"]},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id","content"],"additionalProperties":false}"#,
        "Page write",
        "FileText",
        ["save_page"]
    ),
    action!(
        "page.append",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"blockIds":{"type":"array","items":{"type":"string"}},"before":{"type":"string"},"after":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"targetPageId":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page append",
        "FileText",
        []
    ),
    action!(
        "page.insert",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"blockIds":{"type":"array","items":{"type":"string"}},"before":{"type":"string"},"after":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"targetPageId":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page insert",
        "FileText",
        []
    ),
    action!(
        "page.replace_block",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"blockIds":{"type":"array","items":{"type":"string"}},"before":{"type":"string"},"after":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"targetPageId":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page replace block",
        "FileText",
        []
    ),
    action!(
        "page.delete_blocks",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"blockIds":{"type":"array","items":{"type":"string"}},"before":{"type":"string"},"after":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"targetPageId":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page delete blocks",
        "FileText",
        []
    ),
    action!(
        "page.move_blocks",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"blockIds":{"type":"array","items":{"type":"string"}},"before":{"type":"string"},"after":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"targetPageId":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page move blocks",
        "FileText",
        []
    ),
    action!(
        "page.rename",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id","title"],"additionalProperties":false}"#,
        "Page rename",
        "FileText",
        ["save_page"]
    ),
    action!(
        "page.set_look",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"icon":{"type":"string"},"cover":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id"],"additionalProperties":false}"#,
        "Page set look",
        "FileText",
        ["set_page_look"]
    ),
    action!(
        "page.favorite",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"favorite":{"type":"boolean"}},"required":["id","favorite"],"additionalProperties":false}"#,
        "Page favorite",
        "FileText",
        ["set_page_favorite"]
    ),
    action!(
        "page.move",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"sectionId":{"type":"string"},"parentId":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page move",
        "FileText",
        ["move_page"]
    ),
    action!(
        "page.trash",
        Edit,
        Ask,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Page trash",
        "FileText",
        ["trash_page"]
    ),
    action!(
        "db.trash",
        Edit,
        Ask,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Db trash",
        "Table2",
        ["trash_page"]
    ),
    action!(
        "db.row.trash",
        Edit,
        Ask,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Db row trash",
        "Table2",
        ["trash_page"]
    ),
    action!(
        "block.insert",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"pageId":{"type":"string"},"blockId":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"kind":{"type":"string"},"direction":{"enum":["ltr","rtl","auto"]}},"required":["pageId"],"additionalProperties":false}"#,
        "Block insert",
        "Blocks",
        []
    ),
    action!(
        "block.update",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"pageId":{"type":"string"},"blockId":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"kind":{"type":"string"},"direction":{"enum":["ltr","rtl","auto"]}},"required":["pageId"],"additionalProperties":false}"#,
        "Block update",
        "Blocks",
        []
    ),
    action!(
        "block.turn_into",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"pageId":{"type":"string"},"blockId":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"kind":{"type":"string"},"direction":{"enum":["ltr","rtl","auto"]}},"required":["pageId"],"additionalProperties":false}"#,
        "Block turn into",
        "Blocks",
        []
    ),
    action!(
        "block.set_direction",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"pageId":{"type":"string"},"blockId":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"kind":{"type":"string"},"direction":{"enum":["ltr","rtl","auto"]}},"required":["pageId"],"additionalProperties":false}"#,
        "Block set direction",
        "Blocks",
        []
    ),
    action!(
        "picture.add",
        Edit,
        None,
        History,
        r#"{"type":"object","properties":{"pageId":{"type":"string"},"attachmentId":{"type":"string"}},"required":["pageId","attachmentId"],"additionalProperties":false}"#,
        "Picture add",
        "Image",
        []
    ),
    action!(
        "db.create",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"sectionId":{"type":"string"},"parent":{"type":"string"},"inline":{"type":"boolean"},"title":{"type":"string"},"starterKind":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["sectionId"],"additionalProperties":false}"#,
        "Db create",
        "Table2",
        ["create_database"]
    ),
    action!(
        "db.rename",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"title":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id","title"],"additionalProperties":false}"#,
        "Db rename",
        "Table2",
        ["set_database_look"]
    ),
    action!(
        "db.set_look",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"icon":{"type":"string"},"favorite":{"type":"boolean"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id"],"additionalProperties":false}"#,
        "Db set look",
        "Table2",
        ["set_database_look"]
    ),
    action!(
        "db.move",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"sectionId":{"type":"string"}},"required":["id","sectionId"],"additionalProperties":false}"#,
        "Db move",
        "Table2",
        ["move_database"]
    ),
    action!(
        "db.property.add",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db property add",
        "Table2",
        []
    ),
    action!(
        "db.property.update",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db property update",
        "Table2",
        []
    ),
    action!(
        "db.property.reorder",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db property reorder",
        "Table2",
        []
    ),
    action!(
        "db.property.hide",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db property hide",
        "Table2",
        []
    ),
    action!(
        "db.view.add",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db view add",
        "Table2",
        []
    ),
    action!(
        "db.view.update",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db view update",
        "Table2",
        []
    ),
    action!(
        "db.view.rename",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db view rename",
        "Table2",
        []
    ),
    action!(
        "db.view.reorder",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"value":{"type":"object"},"ids":{"type":"array","items":{"type":"string"}}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db view reorder",
        "Table2",
        []
    ),
    action!(
        "db.property.change_kind",
        Edit,
        Ask,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"kind":{"type":"string"}},"required":["databaseId","id"],"additionalProperties":false}"#,
        "Db property change kind",
        "Table2",
        []
    ),
    action!(
        "db.property.trash",
        Edit,
        Ask,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"kind":{"type":"string"}},"required":["databaseId","id"],"additionalProperties":false}"#,
        "Db property trash",
        "Table2",
        []
    ),
    action!(
        "db.view.trash",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"}},"required":["databaseId","id"],"additionalProperties":false}"#,
        "Db view trash",
        "Table2",
        []
    ),
    action!(
        "db.row.create",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"title":{"type":"string"},"cells":{"type":"object"},"templateId":{"type":"string"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db row create",
        "Table2",
        ["create_row"]
    ),
    action!(
        "db.row.set",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"rowId":{"type":"string"},"title":{"type":"string"},"patch":{"type":"object"},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["rowId"],"additionalProperties":false}"#,
        "Db row set",
        "Table2",
        ["set_cells"]
    ),
    action!(
        "db.row.write_page",
        Edit,
        AskIfBig,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"content":{"oneOf":[{"type":"string"},{"type":"object"},{"type":"array"}]},"format":{"enum":["markdown","blocks"]},"source":{"type":"object","properties":{"app":{"type":"string"},"id":{"type":"string"}},"required":["app","id"],"additionalProperties":false}},"required":["id","content"],"additionalProperties":false}"#,
        "Db row write page",
        "Table2",
        ["save_page"]
    ),
    action!(
        "db.row.duplicate",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"rowId":{"type":"string"}},"required":["rowId"],"additionalProperties":false}"#,
        "Db row duplicate",
        "Table2",
        ["duplicate_row"]
    ),
    action!(
        "db.row.reorder",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}}},"required":["ids"],"additionalProperties":false}"#,
        "Db row reorder",
        "Table2",
        ["reorder_rows"]
    ),
    action!(
        "db.template.list",
        Edit,
        None,
        None,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db template list",
        "Table2",
        []
    ),
    action!(
        "db.template.create",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"template":{"type":"object"}},"required":["databaseId","template"],"additionalProperties":false}"#,
        "Db template create",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.template.update",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"template":{"type":"object"}},"required":["databaseId","template"],"additionalProperties":false}"#,
        "Db template update",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.template.trash",
        Edit,
        Ask,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"}},"required":["databaseId","id"],"additionalProperties":false}"#,
        "Db template trash",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.sub_items.enable",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Db sub items enable",
        "Table2",
        ["enable_sub_items"]
    ),
    action!(
        "db.dependencies.enable",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Db dependencies enable",
        "Table2",
        ["enable_dependencies"]
    ),
    action!(
        "db.automation.list",
        Look,
        None,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db automation list",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.automation.create",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"automation":{"type":"object"}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db automation create",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.automation.update",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"automation":{"type":"object"}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db automation update",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.automation.pause",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"},"automation":{"type":"object"}},"required":["databaseId"],"additionalProperties":false}"#,
        "Db automation pause",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.automation.trash",
        Edit,
        Ask,
        Previous,
        r#"{"type":"object","properties":{"databaseId":{"type":"string"},"id":{"type":"string"}},"required":["databaseId","id"],"additionalProperties":false}"#,
        "Db automation trash",
        "Table2",
        ["save_database"]
    ),
    action!(
        "db.button.press",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"rowId":{"type":"string"},"propId":{"type":"string"}},"required":["rowId","propId"],"additionalProperties":false}"#,
        "Db button press",
        "Table2",
        ["press_button"]
    ),
    action!(
        "db.file.remove",
        Edit,
        Ask,
        Previous,
        r#"{"type":"object","properties":{"rowId":{"type":"string"},"propId":{"type":"string"},"attachmentId":{"type":"string"}},"required":["rowId","propId","attachmentId"],"additionalProperties":false}"#,
        "Db file remove",
        "Table2",
        ["remove_file"]
    ),
    action!(
        "sticky.create",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"open":{"type":"boolean"}},"required":[],"additionalProperties":false}"#,
        "Sticky create",
        "StickyNote",
        ["create_sticky"]
    ),
    action!(
        "sticky.write",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"content":{"type":"string"},"text":{"type":"string"}},"required":["id","content","text"],"additionalProperties":false}"#,
        "Sticky write",
        "StickyNote",
        ["save_sticky"]
    ),
    action!(
        "sticky.style",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"color":{"type":"string"},"onTop":{"type":"boolean"}},"required":["id"],"additionalProperties":false}"#,
        "Sticky style",
        "StickyNote",
        ["set_sticky_style"]
    ),
    action!(
        "sticky.place_on_desktop",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"id":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"},"w":{"type":"number"},"h":{"type":"number"}},"required":["id","x","y","w","h"],"additionalProperties":false}"#,
        "Sticky place on desktop",
        "StickyNote",
        ["set_sticky_geometry"]
    ),
    action!(
        "sticky.open",
        Edit,
        None,
        View,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Sticky open",
        "StickyNote",
        ["open_sticky"]
    ),
    action!(
        "sticky.close",
        Edit,
        None,
        View,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Sticky close",
        "StickyNote",
        ["close_sticky"]
    ),
    action!(
        "sticky.close_all",
        Edit,
        None,
        View,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Sticky close all",
        "StickyNote",
        ["hide_stickies"]
    ),
    action!(
        "calc.add",
        Edit,
        None,
        Previous,
        r#"{"type":"object","properties":{"expr":{"type":"string"},"result":{"type":"string"}},"required":["expr","result"],"additionalProperties":false}"#,
        "Calc add",
        "Calculator",
        ["add_calc_entry"]
    ),
    action!(
        "calc.open",
        Edit,
        None,
        View,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Calc open",
        "Calculator",
        ["open_calculator"]
    ),
    action!(
        "calc.clear",
        Edit,
        Ask,
        Previous,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Calc clear",
        "Calculator",
        ["clear_calc_history"]
    ),
    action!(
        "history.restore",
        Edit,
        Ask,
        History,
        r#"{"type":"object","properties":{"id":{"type":"string"},"versionId":{"type":"string"}},"required":["id","versionId"],"additionalProperties":false}"#,
        "History restore",
        "History",
        []
    ),
    action!(
        "activity.undo",
        Edit,
        AskIfBig,
        Previous,
        r#"{"type":"object","properties":{"changeSetId":{"type":"string"}},"required":["changeSetId"],"additionalProperties":false}"#,
        "Activity undo",
        "List",
        []
    ),
    action!(
        "bin.restore",
        Edit,
        None,
        Bin,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Bin restore",
        "Trash2",
        ["restore_page"]
    ),
    action!(
        "settings.set",
        Full,
        None,
        Previous,
        r#"{"type":"object","properties":{"theme":{"type":"string"},"accent":{"type":"string"},"font":{"type":"string"},"editorWidth":{"type":"string"},"fontSize":{"type":"integer","minimum":12,"maximum":22},"backdrop":{"type":"string"},"density":{"type":"string"},"stickyDefaultColor":{"type":"string"}},"required":[],"additionalProperties":false}"#,
        "Settings set",
        "Settings",
        ["save_settings"]
    ),
    action!(
        "bin.delete_forever",
        Full,
        Strong,
        None,
        r#"{"type":"object","properties":{"id":{"type":"string"}},"required":["id"],"additionalProperties":false}"#,
        "Bin delete forever",
        "Trash2",
        ["delete_page"]
    ),
    action!(
        "bin.empty",
        Full,
        Strong,
        None,
        r#"{"type":"object","properties":{},"required":[],"additionalProperties":false}"#,
        "Bin empty",
        "Trash2",
        ["empty_trash"]
    ),
];
