use std::ops::Range;

pub fn merge<T: AsRef<str>>(edits: &[(Range<usize>, T)]) -> Vec<(Range<usize>, String)> {
    let mut new_edits = Vec::with_capacity(edits.len());
    for i in 0..edits.len() {
        new_edits.push((edits[i].0.clone(), String::from(edits[i].1.as_ref())));
    }
    let mut edits = new_edits;

    // bubble sort, since we can only swap two adjacent edits without making things really
    // complicated
    loop {
        let mut done = true;
        let mut i = 0;
        while i + 1 < edits.len() {
            if edits[i].0.start > edits[i + 1].0.start {
                if edits[i + 1].0.end > edits[i].0.start {
                    // if one edit comes after the other and they do intersect, merge them now,
                    // because if we wait 'till later, we'll overlap things incorrectly
                    if edits[i + 1].0.end < edits[i].0.start + edits[i].1.len() {
                        // in this case, they intersect partially
                        let non_overwritten_start = edits[i + 1].0.end - edits[i].0.start;
                        let removed = edits.remove(i);
                        edits[i].1.push_str(&removed.1[non_overwritten_start..]);
                        edits[i].0.end = removed.0.end;
                    } else {
                        // in this case, the second is a subset of the first and can just be removed
                        let expansion = edits[i].1.len() as i32 - edits[i].0.len() as i32;
                        edits.remove(i);
                        edits[i].0.end = (edits[i].0.end as i32 - expansion) as usize;
                    }
                } else {
                    // if one edit comes after the other and they don't intersect, swap them and
                    // update the range of the second
                    let expansion = edits[i + 1].1.len() as i32 - edits[i + 1].0.len() as i32;
                    edits.swap(i, i + 1);
                    edits[i + 1].0.start = (edits[i + 1].0.start as i32 + expansion) as usize;
                    edits[i + 1].0.end = (edits[i + 1].0.end as i32 + expansion) as usize;
                }
                // if an edit occurs at some point in this loop execution, we're not done sorting
                done = false;
            }
            i += 1;
        }
        if done {
            break;
        }
    }

    // merging
    let mut i = 0;
    while i + 1 < edits.len() {
        // if the end of the text updated by the first edit overlaps the start of the next, merge
        // them
        if edits[i].0.start + edits[i].1.len() > edits[i + 1].0.start {
            if edits[0].0.start + edits[i].1.len() >= edits[i + 1].0.end {
                // in this case, the area updated by the second is a subset of that updated by the
                // first
                let mut new_text =
                    String::from(&edits[i].1[..edits[i + 1].0.start - edits[i].0.start]);
                new_text.push_str(&edits[i + 1].1);
                new_text.push_str(&edits[i].1[edits[i + 1].0.end - edits[i].0.start..]);
                edits[i].1 = new_text;
                edits.remove(i + 1);
            } else {
                // in this case, they intersect partially
                let expansion = edits[i].1.len() as i32 - edits[i].0.len() as i32;
                let mut new_text =
                    String::from(&edits[i].1[..edits[i + 1].0.start - edits[i].0.start]);
                new_text.push_str(&edits[i + 1].1);
                edits[i].1 = new_text;
                edits[i].0.end = (edits[i + 1].0.end as i32 - expansion) as usize;
                edits.remove(i + 1);
            }
        }
        i += 1;
    }
    edits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply<T: AsRef<str>>(text: &str, edits: &[(Range<usize>, T)]) -> String {
        let mut prev = String::from(text);
        for edit in edits {
            let mut next = String::from(&prev[..edit.0.start]);
            next.push_str(edit.1.as_ref());
            next.push_str(&prev[edit.0.end..]);
            prev = next;
        }
        prev
    }

    fn sorted(edits: &Vec<(Range<usize>, String)>) -> bool {
        for i in 0..edits.len() - 1 {
            if edits[i].0.start > edits[i + 1].0.start {
                return false;
            }
        }
        true
    }

    fn non_overlapping(edits: &Vec<(Range<usize>, String)>) -> bool {
        for i in 0..edits.len() - 1 {
            if edits[i].0.start + edits[i].1.len() > edits[i + 1].0.start {
                return false;
            }
        }
        true
    }

    #[test]
    fn non_overlapping_misordered() {
        let text = String::from("hello world");
        let edits = vec![(6..11, "earth"), (0..5, "hi")];
        assert_eq!("hi earth", apply(&text, &edits));

        let new_edits = merge(&edits);
        assert_eq!("hi earth", apply(&text, &new_edits));
    }

    #[test]
    fn non_overlapping_ordered() {
        let text = String::from("hello world");
        let edits = vec![(0..5, "hi"), (3..8, "earth")];
        assert_eq!("hi earth", apply(&text, &edits));

        let new_edits = merge(&edits);
        assert_eq!("hi earth", apply(&text, &new_edits));
        assert!(sorted(&new_edits));
        assert!(non_overlapping(&new_edits));
    }

    #[test]
    fn intersecting_misordered() {
        let text = String::from("hello to the world");
        let edits = vec![(9..18, "big earth"), (0..12, "hi")];
        assert_eq!("hi earth", apply(&text, &edits));

        let new_edits = merge(&edits);
        assert_eq!("hi earth", apply(&text, &new_edits));
        assert!(sorted(&new_edits));
        assert!(non_overlapping(&new_edits));
    }

    #[test]
    fn intersecting_ordered() {
        let text = String::from("hello to the world");
        let edits = vec![(0..12, "hello big"), (6..15, "earth")];
        assert_eq!("hello earth", apply(&text, &edits));

        let new_edits = merge(&edits);
        assert_eq!("hello earth", apply(&text, &new_edits));
        assert!(sorted(&new_edits));
        assert!(non_overlapping(&new_edits));
    }

    #[test]
    fn subset_misordered() {
        let text = String::from("hello big world");
        let edits = vec![(6..9, "small"), (0..17, "rewrite")];
        assert_eq!("rewrite", apply(&text, &edits));

        let new_edits = merge(&edits);
        assert_eq!("rewrite", apply(&text, &new_edits));
        assert!(sorted(&new_edits));
        assert!(non_overlapping(&new_edits));
    }

    #[test]
    fn subset_ordered() {
        let text = String::from("hello to the world");
        let edits = vec![(0..18, "hi big earth"), (3..7, "")];
        assert_eq!("hi earth", apply(&text, &edits));

        let new_edits = merge(&edits);
        assert_eq!("hi earth", apply(&text, &new_edits));
        assert!(sorted(&new_edits));
        assert!(non_overlapping(&new_edits));
    }
}
