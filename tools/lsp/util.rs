// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

use i_slint_compiler::diagnostics::{DiagnosticLevel, Spanned};
use i_slint_compiler::langtype::ElementType;
use i_slint_compiler::lookup::LookupCtx;
use i_slint_compiler::object_tree;
use i_slint_compiler::parser::{syntax_nodes, SyntaxKind, SyntaxNode};
use i_slint_compiler::typeregister::TypeRegister;

use crate::DocumentCache;

/// Given a node within an element, return the Type for the Element under that node.
/// (If node is an element, return the Type for that element, otherwise the type of the element under it)
/// Will return `Foo` in the following example where `|` is the cursor.
///
/// ```text
/// Hello := A {
///   B {
///      Foo {
///        |
///      }
///   }
/// }
/// ```
pub fn lookup_current_element_type(mut node: SyntaxNode, tr: &TypeRegister) -> Option<ElementType> {
    while node.kind() != SyntaxKind::Element {
        if let Some(parent) = node.parent() {
            node = parent
        } else {
            return None;
        }
    }
    let parent = node
        .parent()
        .and_then(|parent| lookup_current_element_type(parent, tr))
        .unwrap_or_default();
    let qualname = object_tree::QualifiedTypeName::from_node(
        syntax_nodes::Element::from(node).QualifiedName()?,
    );
    parent.lookup_type_for_child_element(&qualname.to_string(), tr).ok()
}

/// Run the function with the LookupCtx associated with the token
pub fn with_lookup_ctx<R>(
    document_cache: &DocumentCache,
    node: SyntaxNode,
    f: impl FnOnce(&mut LookupCtx) -> R,
) -> Option<R> {
    let (element, prop_name) = lookup_expression_context(node.clone())?;
    with_property_lookup_ctx::<R>(document_cache, &element, &prop_name, f)
}

/// Run the function with the LookupCtx associated with the token
pub fn with_property_lookup_ctx<R>(
    document_cache: &DocumentCache,
    element: &syntax_nodes::Element,
    prop_name: &str,
    f: impl FnOnce(&mut LookupCtx) -> R,
) -> Option<R> {
    let global_tr = document_cache.documents.global_type_registry.borrow();
    let tr = element
        .source_file()
        .and_then(|sf| document_cache.documents.get_document(sf.path()))
        .map(|doc| &doc.local_registry)
        .unwrap_or(&global_tr);

    let ty = element
        .PropertyDeclaration()
        .find_map(|p| {
            (i_slint_compiler::parser::identifier_text(&p.DeclaredIdentifier())? == prop_name)
                .then(|| p)
        })
        .and_then(|p| p.Type())
        .map(|n| object_tree::type_from_node(n, &mut Default::default(), tr))
        .or_else(|| {
            lookup_current_element_type((**element).clone(), tr)
                .map(|el_ty| el_ty.lookup_property(&prop_name).property_type)
        });

    // FIXME: we need also to fill in the repeated element
    let component = {
        let mut n = element.parent()?;
        loop {
            if let Some(component) = syntax_nodes::Component::new(n.clone()) {
                break component;
            }
            n = n.parent()?;
        }
    };

    let component = i_slint_compiler::parser::identifier_text(&component.DeclaredIdentifier())
        .and_then(|component_name| tr.lookup_element(&component_name).ok())?;
    let scope = if let ElementType::Component(c) = component {
        vec![c.root_element.clone()]
    } else {
        Vec::new()
    };

    let mut build_diagnostics = Default::default();
    let mut lookup_context = LookupCtx::empty_context(tr, &mut build_diagnostics);
    lookup_context.property_name = Some(&prop_name);
    lookup_context.property_type = ty.unwrap_or_default();
    lookup_context.component_scope = &scope;
    Some(f(&mut lookup_context))
}

/// Return the element and property name in which we are
fn lookup_expression_context(mut n: SyntaxNode) -> Option<(syntax_nodes::Element, String)> {
    let (element, prop_name) = loop {
        if let Some(decl) = syntax_nodes::PropertyDeclaration::new(n.clone()) {
            let prop_name = i_slint_compiler::parser::identifier_text(&decl.DeclaredIdentifier())?;
            let element = syntax_nodes::Element::new(n.parent()?)?;
            break (element, prop_name);
        }
        match n.kind() {
            SyntaxKind::Binding
            | SyntaxKind::TwoWayBinding
            // FIXME: arguments of the callback
            | SyntaxKind::CallbackConnection => {
                let prop_name = i_slint_compiler::parser::identifier_text(&n)?;
                let element = syntax_nodes::Element::new(n.parent()?)?;
                break (element, prop_name);
            }
            SyntaxKind::ConditionalElement | SyntaxKind::RepeatedElement => {
                let element = syntax_nodes::Element::new(n.parent()?)?;
                break (element, "$model".to_string());
            }
            SyntaxKind::Element => {
                // oops: missed it
                let element = syntax_nodes::Element::new(n)?;
                break (element, String::new());
            }
            _ => n = n.parent()?,

        }
    };
    Some((element, prop_name))
}

pub fn to_lsp_diag(d: &i_slint_compiler::diagnostics::Diagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic::new(
        to_range(d.line_column()),
        Some(to_lsp_diag_level(d.level())),
        None,
        None,
        d.message().to_owned(),
        None,
        None,
    )
}

fn to_range(span: (usize, usize)) -> lsp_types::Range {
    let pos = lsp_types::Position::new(
        (span.0 as u32).saturating_sub(1),
        (span.1 as u32).saturating_sub(1),
    );
    lsp_types::Range::new(pos, pos)
}

fn to_lsp_diag_level(level: DiagnosticLevel) -> lsp_types::DiagnosticSeverity {
    match level {
        DiagnosticLevel::Error => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticLevel::Warning => lsp_types::DiagnosticSeverity::WARNING,
        _ => lsp_types::DiagnosticSeverity::INFORMATION,
    }
}
