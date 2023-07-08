use cli_table::{Cell, Style, Table};
pub(crate) fn to_table(krate: &super::Crate) -> String {
    let mut output = krate
        .traits
        .iter()
        .map(|t| {
            vec![
                "trait".cell(),
                format!("{}::{}", t.path, t.name).cell(),
                t.decl.clone().cell(),
                t.has_generics.cell(),
                t.stability.cell(),
            ]
        })
        .collect::<Vec<_>>();

    output.append(
        &mut krate
            .structs
            .iter()
            .map(|t| {
                vec![
                    "struct".cell(),
                    format!("{}::{}", t.path, t.name).cell(),
                    t.decl.clone().cell(),
                    t.has_generics.cell(),
                    t.stability.cell(),
                ]
            })
            .collect::<Vec<_>>(),
    );

    output.append(
        &mut krate
            .enums
            .iter()
            .map(|t| {
                vec![
                    "enums".cell(),
                    format!("{}::{}", t.path, t.name).cell(),
                    t.decl.clone().cell(),
                    t.has_generics.cell(),
                    t.stability.cell(),
                ]
            })
            .collect::<Vec<_>>(),
    );

    output.append(
        &mut krate
            .functions
            .iter()
            .map(|t| {
                vec![
                    "function".cell(),
                    format!("{}::{}", t.path, t.name).cell(),
                    t.decl.clone().cell(),
                    t.has_generics.cell(),
                    t.stability.cell(),
                ]
            })
            .collect::<Vec<_>>(),
    );
    output
        .table()
        .title(vec![
            "Kind".cell().bold(true),
            "Name".cell().bold(true),
            "Signature".cell().bold(true),
            "Generics?".cell().bold(true),
            "Stability".cell().bold(true),
        ])
        .display()
        .unwrap()
        .to_string()
}
