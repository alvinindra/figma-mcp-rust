//! Merge multiple single-page PDFs (returned by the plugin as base64) into one
//! multi-page PDF using `lopdf`. Mirrors the Go `mergePDFPages` helper.

use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object, ObjectId};

/// Merge one or more PDF byte slices into a single PDF document.
pub fn merge_pdfs(pages: &[Vec<u8>]) -> Result<Vec<u8>> {
    if pages.is_empty() {
        return Err(anyhow!("no pages to merge"));
    }
    if pages.len() == 1 {
        return Ok(pages[0].clone());
    }

    let mut docs: Vec<Document> = Vec::with_capacity(pages.len());
    for (i, raw) in pages.iter().enumerate() {
        docs.push(
            Document::load_mem(raw).with_context(|| format!("frame {i}: failed to parse PDF"))?,
        );
    }

    let mut max_id = 1u32;
    let mut pages_object: Option<(ObjectId, Object)> = None;
    let mut documents_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut output = Document::with_version("1.5");

    for mut doc in docs {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        let pages_iter = doc
            .get_pages()
            .into_values()
            .map(|oid| (oid, doc.get_object(oid).cloned().unwrap_or(Object::Null)));
        documents_pages.extend(pages_iter);
        documents_objects.extend(doc.objects);
    }

    // Find the Catalog and Pages objects from the assembled set.
    let mut catalog_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in &documents_objects {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object = Some((*object_id, object.clone()));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, existing)) = pages_object.as_ref() {
                        if let Ok(prev) = existing.as_dict() {
                            dictionary.extend(prev);
                        }
                    }
                    pages_object = Some((
                        pages_object
                            .as_ref()
                            .map(|(id, _)| *id)
                            .unwrap_or(*object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            _ => {}
        }
    }

    let (pages_id, pages_obj) = pages_object.ok_or_else(|| anyhow!("no Pages object found"))?;

    // Update every page to point at the merged Pages object.
    for (object_id, object) in &mut documents_pages {
        if let Ok(dictionary) = object.as_dict_mut() {
            dictionary.set("Parent", pages_id);
            documents_objects.insert(*object_id, object.clone());
        }
    }

    // Build new pages dictionary
    let kids: Vec<Object> = documents_pages
        .keys()
        .map(|id| Object::Reference(*id))
        .collect();
    let count = kids.len() as i64;
    let mut pages_dict = pages_obj.as_dict().cloned().unwrap_or_default();
    pages_dict.set("Kids", Object::Array(kids));
    pages_dict.set("Count", count);
    pages_dict.set("Type", "Pages");
    documents_objects.insert(pages_id, Object::Dictionary(pages_dict));

    // Update the Catalog to use the merged Pages.
    let (catalog_id, catalog_object) = catalog_object.ok_or_else(|| anyhow!("no Catalog found"))?;
    if let Ok(dict) = catalog_object.as_dict() {
        let mut dict = dict.clone();
        dict.set("Pages", pages_id);
        documents_objects.insert(catalog_id, Object::Dictionary(dict));
    }

    output.objects = documents_objects;
    output.trailer.set("Root", catalog_id);
    output.max_id = max_id;
    output.compress();

    let mut buf = Vec::new();
    output
        .save_to(&mut buf)
        .map_err(|e| anyhow!("save merged PDF: {e}"))?;
    Ok(buf)
}
