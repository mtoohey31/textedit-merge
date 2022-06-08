use std::ops::Range;

pub fn merge<T: AsRef<str>>(
    edits1: &[(Range<usize>, T)],
    edits2: &[(Range<usize>, T)],
) -> Vec<(Range<usize>, String)> {
    let (mut i, mut j) = (0, 0);
    let mut edits1 = edits1
        .iter()
        .map(|(r, s)| (r.clone(), s.as_ref().to_string()))
        .collect::<Vec<_>>();
    let mut edits2 = edits2
        .iter()
        .map(|(r, s)| (r.clone(), s.as_ref().to_string()))
        .collect::<Vec<_>>();
    let mut res = Vec::new();
    while i < edits1.len() && j < edits2.len() {
        if edits1[i].0.start <= edits2[j].0.start {
            // the next edits1 element comes first

            // TODO: do something fancy so this isn't O(n^2)
            let expansion = edits1[i].1.len() as isize - edits1[i].0.len() as isize;
            for k in j..edits2.len() {
                edits2[k].0.start = (edits2[k].0.start as isize - expansion) as usize;
                edits2[k].0.end = (edits2[k].0.end as isize - expansion) as usize;
            }
            if edits1[i].0.end <= edits2[j].0.start {
                // in this case, the edits are completely non-overlapping, so we just move the one
                // along, and adjust all following elements of edits2 by the expansion
                res.push(edits1[i].clone());
                i += 1;
            } else {
                // in this case, they overlap in some way. edits2[j] happens second, so we merge
                // things as such
                let mut new: String;
                if edits2[j].0.end < edits1[i].0.end {
                    // in this case, the range updated by edits2[j] is a subset of that updated by
                    // edits1, so we have to take both start and end parts from edits1's insert
                    let from1_start = &edits1[i].1[..(edits2[j].0.start as isize + expansion
                        - edits1[i].0.start as isize)
                        as usize];
                    let from1_end = &edits1[i].1[(edits2[j].0.end as isize + expansion
                        - edits1[i].0.start as isize)
                        as usize..];
                    new = String::with_capacity(
                        from1_start.len() + edits2[j].1.len() + from1_end.len(),
                    );
                    new.push_str(from1_start);
                    new.push_str(&edits2[j].1);
                    new.push_str(from1_end);
                } else {
                    // in this case they overlap normally, so we take a part from the start of
                    // edits1[i], along with the all of edits2[j]
                    let from1 = &edits1[i].1[..(edits2[j].0.start as isize + expansion
                        - edits1[i].0.start as isize)
                        as usize];
                    new = String::with_capacity(from1.len() + edits2[j].1.len());
                    new.push_str(from1);
                    new.push_str(&edits2[j].1);
                    edits1[i].0.end = edits2[j].0.end;
                }
                // we update edits1[i] here instead of just incrementing i and pushing to res
                // because edits1[i] may intersect with more than one item in edits2
                edits1[i].1 = new;
                j += 1;
            }
        } else {
            // the next edits2 element comes first
            if edits1[i].0.start >= edits2[j].0.end {
                // in this case, the edits are also completely non-overlapping, so we also move the
                // one along, and there's no need to adjust anything by expansion because edits in
                // edits2 happen after those in edits1
                res.push(edits2[j].clone());
                j += 1;
            } else {
                // in this case, they overlap in some way. edits1[i] happens second, so we merge things as such
                if edits1[i].0.end < edits2[j].0.end {
                    // in this case, the area updated by edits1[i] is a subset of the area updated
                    // by edits2[j], so we just update the end of edits2[j] (since it will overwrite
                    // all of the text written by edits1[i]) by the expansion factor of edits1[i]
                    let expansion = edits1[i].1.len() as isize - edits1[i].0.len() as isize;
                    edits2[j].0.end = (edits2[j].0.end as isize - expansion) as usize;
                } else {
                    // in this case, they overlap normally, so we take all of edits2[j]'s insert
                    // and part of edits1[i]'s
                    let from1 = &edits1[i].1[edits2[j].0.end - edits1[i].0.start..];
                    let mut new = String::with_capacity(from1.len() + edits2[j].1.len());
                    new.push_str(&edits2[j].1);
                    new.push_str(&from1);
                    edits2[j].0.end = edits1[i].0.end;
                    edits2[j].1 = new;
                }
                i += 1;
            }
        }
    }
    res.extend_from_slice(&edits1[i..]);
    res.extend_from_slice(&edits2[j..]);
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply<T: AsRef<str>>(text: &str, edits: &[(Range<usize>, T)]) -> String {
        let mut len = text.len();
        for (r, s) in edits {
            len += s.as_ref().len();
            len -= r.len();
        }

        let mut res = String::with_capacity(len);
        let mut prev = 0;
        for (r, s) in edits {
            res.push_str(&text[prev..r.start]);
            res.push_str(s.as_ref());
            prev = r.end;
        }
        res.push_str(&text[prev..]);
        res
    }

    #[test]
    fn non_overlapping_1_2() {
        let text = String::from("hello world");
        let edits1 = vec![(0..5, "hi")];
        let edits2 = vec![(3..8, "earth")];
        assert_eq!("hi earth", apply(&apply(&text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!("hi earth", apply(&text, &new_edits));
    }

    #[test]
    fn non_overlapping_2_1() {
        let text = String::from("hello world");
        let edits1 = vec![(6..11, "earth")];
        let edits2 = vec![(0..5, "hi")];
        assert_eq!("hi earth", apply(&apply(&text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!("hi earth", apply(&text, &new_edits));
    }

    #[test]
    fn overlapping_1_2() {
        let text = String::from("hello to the world");
        let edits1 = vec![(0..12, "hi there")];
        let edits2 = vec![(3..14, "earth")];
        assert_eq!("hi earth", apply(&apply(&text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!("hi earth", apply(&text, &new_edits));
    }

    #[test]
    fn overlapping_2_1() {
        let text = String::from("hello to the world");
        let edits1 = vec![(9..18, "big earth")];
        let edits2 = vec![(0..12, "hi")];
        assert_eq!("hi earth", apply(&apply(&text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!("hi earth", apply(&text, &new_edits));
    }

    #[test]
    fn subset_1_2() {
        let text = String::from("hello big earth");
        let edits1 = vec![(0..15, "hi big world")];
        let edits2 = vec![(3..6, "small")];
        assert_eq!("hi small world", apply(&apply(&text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!("hi small world", apply(&text, &new_edits));
    }

    #[test]
    fn subset_2_1() {
        let text = String::from("hello big earth");
        let edits1 = vec![(6..9, "small")];
        let edits2 = vec![(0..17, "hi world")];
        assert_eq!("hi world", apply(&apply(&text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        dbg!(&new_edits);
        assert_eq!("hi world", apply(&text, &new_edits));
    }
}
