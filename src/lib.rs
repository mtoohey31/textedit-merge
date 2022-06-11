use std::ops::Range;

const TEXT: &str = r#"{
  # how to string-format the option name;
  # by default one character is a short option (`-`),
  # more than one characters a long option (`--`).
  mkOptionName ?
  k: if builtins.stringLength k == 1
    then "-${k}"
    else "--${k}"
, mkOption ? k: v:
  if v == null
  then [ ]
  else [ (mkOptionName k) (lib.generators.mkValueStringDefault { } v) ]
}:
{  toINI = {
  # parameter comment
  mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
  , mkKeyValue ? mkKeyValueDefault {} "="
  }: attrsOfAttrs:
    mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;
}
"#;

const EXPECTED: &str = r#"{
  # how to string-format the option name;
  # by default one character is a short option (`-`),
  # more than one characters a long option (`--`).
  mkOptionName ? k:
    if builtins.stringLength k == 1
    then "-${k}"
    else "--${k}"
, mkOption ? k: v:
    if v == null
    then [ ]
    else [ (mkOptionName k) (lib.generators.mkValueStringDefault { } v) ]
}:
{
  toINI =
    {
      # parameter comment
      mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
    , mkKeyValue ? mkKeyValueDefault { } "="
    }: attrsOfAttrs:
    mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;
}
"#;

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
    let mut res: Vec<(Range<usize>, String)> = Vec::new();

    let mut total_expansion_so_far = 0;
    while i < edits1.len() && j < edits2.len() {
        #[cfg(test)]
        dbg!(edits1[i].0.start, edits2[j].0.start, total_expansion_so_far);

        if edits1[i].0.start < (edits2[j].0.start as isize - total_expansion_so_far) as usize {
            // the next edits1 element comes first
            let expansion = edits1[i].1.len() as isize - edits1[i].0.len() as isize;

            if edits1[i].0.end
                <= (edits2[j].0.start as isize - total_expansion_so_far - expansion) as usize
            {
                // in this case, the edits are completely non-overlapping, so we just move the one
                // along, and adjust all following elements of edits2 by the expansion
                total_expansion_so_far += edits1[i].1.len() as isize;
                total_expansion_so_far -= edits1[i].0.len() as isize;
                res.push(edits1[i].clone());
                i += 1;

                #[cfg(test)]
                dbg!("12dj");
            } else {
                // in this case, they overlap in some way. edits2[j] happens second, so we merge
                // things as such
                if edits2[j].0.end < edits1[i].0.end {
                    // in this case, the range updated by edits2[j] is a subset of that updated by
                    // edits1, so we have to take both start and end parts from edits1's insert
                    let from1_start = &edits1[i].1
                        [..(edits2[j].0.start as isize - edits1[i].0.start as isize) as usize];
                    let from1_end = &edits1[i].1
                        [(edits2[j].0.end as isize - edits1[i].0.start as isize) as usize..];
                    let mut new = String::with_capacity(
                        from1_start.len() + edits2[j].1.len() + from1_end.len(),
                    );
                    new.push_str(from1_start);
                    new.push_str(&edits2[j].1);
                    new.push_str(from1_end);
                    edits1[i].1 = new;

                    // TODO: find something to test this, cause I think I need to adjust more stuff
                    // here

                    #[cfg(test)]
                    dbg!("12ss");
                } else {
                    // in this case they overlap normally, so we take a part from the start of
                    // edits1[i], along with the all of edits2[j]
                    let from1 = &edits1[i].1
                        [..(edits2[j].0.start as isize - edits1[i].0.start as isize) as usize];
                    let mut new = String::with_capacity(from1.len() + edits2[j].1.len());
                    new.push_str(from1);
                    new.push_str(&edits2[j].1);
                    edits1[i].0.end = (edits2[j].0.end as isize - expansion) as usize;

                    let expansion = new.len() as isize - edits1[i].1.len() as isize;
                    for k in j..edits2.len() {
                        edits2[k].0.start = (edits2[k].0.start as isize + expansion) as usize;
                        edits2[k].0.end = (edits2[k].0.end as isize + expansion) as usize;
                    }
                    edits1[i].1 = new;

                    #[cfg(test)]
                    dbg!("12ov");
                }
                j += 1;
            }
        } else {
            // the next edits2 element comes first
            if edits1[i].0.start >= (edits2[j].0.end as isize - total_expansion_so_far) as usize {
                // in this case, the edits are also completely non-overlapping, so we also move the
                // one along, and there's no need to adjust anything by expansion because edits in
                // edits2 happen after those in edits1
                let expansion = edits2[j].1.len() as isize - edits2[j].0.len() as isize;
                for k in j + 1..edits2.len() {
                    edits2[k].0.start = (edits2[k].0.start as isize + expansion) as usize;
                    edits2[k].0.end = (edits2[k].0.end as isize + expansion) as usize;
                }
                edits2[j].0.start = (edits2[j].0.start as isize - total_expansion_so_far) as usize;
                edits2[j].0.end = (edits2[j].0.end as isize - total_expansion_so_far) as usize;

                total_expansion_so_far += edits2[j].1.len() as isize;
                total_expansion_so_far -= edits2[j].0.len() as isize;
                res.push(edits2[j].clone());
                j += 1;

                #[cfg(test)]
                dbg!("21dj");
            } else {
                // in this case, they overlap in some way. edits1[i] happens second, so we merge things as such
                if edits1[i].0.start + edits1[i].1.len() <= edits2[j].0.end {
                    // in this case, the area updated by edits1[i] is a subset of the area updated
                    // by edits2[j], so we just update the end of edits2[j] (since it will overwrite
                    // all of the text written by edits1[i]) by the expansion factor of edits1[i]
                    let expansion = edits1[i].1.len() as isize - edits1[i].0.len() as isize;
                    edits2[j].0.end = (edits2[j].0.end as isize - expansion) as usize;

                    let expansion = edits1[i].1.len() as isize - edits1[i].0.len() as isize;
                    for k in j + 1..edits2.len() {
                        edits2[k].0.start = (edits2[k].0.start as isize - expansion) as usize;
                        edits2[k].0.end = (edits2[k].0.end as isize - expansion) as usize;
                    }

                    #[cfg(test)]
                    dbg!("21ss");
                } else {
                    dbg!(&edits1[i], &edits2[j]);

                    // in this case, they overlap normally, so we take all of edits2[j]'s insert
                    // and part of edits1[i]'s
                    let from1 = &edits1[i].1[(edits2[j].0.end as isize
                        - total_expansion_so_far
                        - edits1[i].0.start as isize)
                        as usize..];
                    let mut new = String::with_capacity(from1.len() + edits2[j].1.len());
                    new.push_str(&edits2[j].1);
                    new.push_str(&from1);
                    edits2[j].0.end = (edits1[i].0.end as isize + total_expansion_so_far) as usize;

                    let expansion = new.len() as isize - edits2[j].1.len() as isize;
                    for k in j + 1..edits2.len() {
                        edits2[k].0.start = (edits2[k].0.start as isize - expansion) as usize;
                        edits2[k].0.end = (edits2[k].0.end as isize - expansion) as usize;
                    }

                    edits2[j].1 = new;

                    #[cfg(test)]
                    dbg!("21ov");
                }
                i += 1;
            }
        }

        // #[cfg(test)]
        // if edits1.len() > 1 {
        //     let mut combined = res.clone();
        //     combined.extend_from_slice(&edits1[i..]);
        //     dbg!(i, j, &edits1[i..], &edits2[j..], &res);
        //     let intermediate = tests::apply(&TEXT, &combined);
        //     dbg!(&intermediate);
        //     assert_eq!(EXPECTED, tests::apply(&intermediate, &edits2[j..]));
        // }
    }
    res.extend_from_slice(&edits1[i..]);
    for (r, s) in edits2[j..].into_iter() {
        let mut r = r.clone();
        r.start = (r.start as isize - total_expansion_so_far) as usize;
        r.end = (r.end as isize - total_expansion_so_far) as usize;
        res.push((r, s.clone()));
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn apply<T: AsRef<str>>(text: &str, edits: &[(Range<usize>, T)]) -> String {
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
        let text = "hello world";
        let expected = "hi earth";
        let edits1 = vec![(0..5, "hi")];
        let edits2 = vec![(3..8, "earth")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn non_overlapping_2_1() {
        let text = "hello world";
        let expected = "hi earth";
        let edits1 = vec![(6..11, "earth")];
        let edits2 = vec![(0..5, "hi")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn overlapping_1_2() {
        let text = "hello to the world";
        let expected = "hi earth";
        let edits1 = vec![(0..12, "hi there")];
        let edits2 = vec![(3..14, "earth")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn overlapping_2_1() {
        let text = "hello to the world";
        let expected = "hi earth";
        let edits1 = vec![(9..18, "big earth")];
        let edits2 = vec![(0..12, "hi")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn subset_1_2() {
        let text = "hello big earth";
        let expected = "hi small world";
        let edits1 = vec![(0..15, "hi big world")];
        let edits2 = vec![(3..6, "small")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn subset_2_1() {
        let text = "hello big earth";
        let expected = "hi world";
        let edits1 = vec![(6..9, "small")];
        let edits2 = vec![(0..17, "hi world")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn attr_fn() {
        let text = r#"{
  f = { x
  , y
      }: body;

  testAllTrue = expr: {inherit expr;expected=map (x: true) expr; };
}
    "#;
        let expected = r#"{
  f =
    { x
    , y
    }: body;

  testAllTrue = expr: { inherit expr; expected = map (x: true) expr; };
}
    "#;
        let edits1 = vec![
            (7..8, "\n"),
            (57..57, " "),
            (70..70, " "),
            (78..78, " "),
            (79..79, " "),
        ];
        let edits2 = vec![(7..8, "\n    "), (11..14, "\n    "), (17..24, "\n    ")];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn curried_fn() {
        let text = r#"{
mkValueStringDefault = {}: v: with builtins;
let err = t: v: abort
"oups";
in 92;
}
"#;

        let expected = r#"{
  mkValueStringDefault = {}: v: with builtins;
    let
      err = t: v: abort
        "oups";
    in
    92;
}
"#;

        let edits1 = vec![(50..51, "\n"), (79..80, "\n")];
        let edits2 = vec![
            (1..2, "\n  "),
            (46..47, "\n    "),
            (50..51, "\n      "),
            (68..69, "\n        "),
            (76..77, "\n    "),
            (79..80, "\n    "),
        ];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn existing_blank_lines() {
        let text = r#"[1

# first block
2
3

# second block
4
5]
"#;

        let expected = r#"[
  1

  # first block
  2
  3

  # second block
  4
  5
]
"#;
        let edits1 = vec![(1..1, "\n"), (41..41, "\n")];
        let edits2 = vec![
            (1..2, "\n  "),
            (3..5, "\n\n  "),
            (18..19, "\n  "),
            (20..21, "\n  "),
            (22..24, "\n\n  "),
            (38..39, "\n  "),
            (40..41, "\n  "),
        ];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn indent_lambda_top_level() {
        let text = r#"import ./make-test-python.nix ({pkgs, lib, ...}:

let
  bar = 57;
in {
  baz = qux;
})"#;

        let expected = r#"import ./make-test-python.nix ({ pkgs, lib, ... }:

let
  bar = 57;
in
{
  baz = qux;
})
"#;
        let edits1 = vec![(32..32, " "), (46..46, " "), (68..69, "\n"), (86..86, "\n")];
        let edits2 = vec![];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn indent_tabs_2() {
        let text = r#"args@{	z	,	}	:

		  	{
		a	 =  	 	"1"	;
		b	 =	 {	} 	// 	{	}	;		
		c 	=	   	z	.	y	;
		d 	=	 1.2	;	
	e 	=	   5	  	;
		f	 = 	./test		;
			g	 = 	true	;		
		h 		=	 null	;
	i 	  		=	 [	 1	 ./example.bin	 { 		hello	=		"world"	;	 } 	]	;	
			j 	= 		5	 * 			5;			
	k     = z ? fs		    ;
	l   = z 		++		 z;
	m 	=	 - 324		;
	n = z		  z    ;	
	o = import 	 	 	 		<nixpkgs>		;
	p = with z; "fdsfs";			    
	q = ''
  ${5+5}
	'';
	r = " f${z}ds ";
		  	  		 }"#;

        let expected = r#"args@{ z, }:

{
  a = "1";
  b = { } // { };
  c = z.y;
  d = 1.2;
  e = 5;
  f = ./test;
  g = true;
  h = null;
  i = [ 1 ./example.bin { hello = "world"; } ];
  j = 5 * 5;
  k = z ? fs;
  l = z ++ z;
  m = - 324;
  n = z z;
  o = import <nixpkgs>;
  p = with z; "fdsfs";
  q = ''
      ${5+5}
    	'';
  r = " f${z}ds ";
}
"#;
        let edits1 = vec![
            (6..7, " "),
            (8..9, ""),
            (10..11, " "),
            (12..13, ""),
            (26..28, " "),
            (29..34, " "),
            (37..38, ""),
            (43..45, " "),
            (46..48, " "),
            (49..50, " "),
            (51..53, " "),
            (55..57, " "),
            (58..59, " "),
            (60..61, ""),
            (68..70, " "),
            (71..76, " "),
            (77..78, ""),
            (79..80, ""),
            (81..82, ""),
            (87..89, " "),
            (90..92, " "),
            (95..96, ""),
            (101..103, " "),
            (104..108, " "),
            (109..113, ""),
            (118..120, " "),
            (121..123, " "),
            (129..131, ""),
            (137..139, " "),
            (140..142, " "),
            (146..147, ""),
            (154..157, " "),
            (158..160, " "),
            (164..165, ""),
            (169..175, " "),
            (176..178, " "),
            (179..181, " "),
            (182..184, " "),
            (197..199, " "),
            (200..203, " "),
            (208..209, " "),
            (210..212, " "),
            (219..220, ""),
            (221..223, " "),
            (224..226, " "),
            (227..228, ""),
            (235..237, " "),
            (238..241, " "),
            (242..244, " "),
            (245..249, " "),
            (257..262, " "),
            (270..276, ""),
            (280..283, " "),
            (286..289, " "),
            (291..294, " "),
            (299..301, " "),
            (302..304, " "),
            (309..311, ""),
            (319..323, " "),
            (324..328, ""),
            (342..351, " "),
            (360..362, ""),
            (444..444, "\n"),
        ];
        let edits2 = vec![
            (12..19, "\n\n"),
            (20..23, "\n  "),
            (31..34, "\n  "),
            (49..54, "\n  "),
            (62..65, "\n  "),
            (73..76, "\n  "),
            (82..85, "\n  "),
            (96..100, "\n  "),
            (109..114, "\n  "),
            (123..125, "\n  "),
            (170..175, "\n  "),
            (185..190, "\n  "),
            (201..203, "\n  "),
            (214..216, "\n  "),
            (226..228, "\n  "),
            (236..239, "\n  "),
            (260..262, "\n  "),
            (282..291, "\n  "),
            (298..298, "    "),
            (307..307, "    "),
            (311..313, "\n  "),
            (329..340, "\n"),
        ];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }

    #[test]
    fn indented_lambda() {
        let text = r#"{
  # how to string-format the option name;
  # by default one character is a short option (`-`),
  # more than one characters a long option (`--`).
  mkOptionName ?
  k: if builtins.stringLength k == 1
    then "-${k}"
    else "--${k}"
, mkOption ? k: v:
  if v == null
  then [ ]
  else [ (mkOptionName k) (lib.generators.mkValueStringDefault { } v) ]
}:
{  toINI = {
  # parameter comment
  mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
  , mkKeyValue ? mkKeyValueDefault {} "="
  }: attrsOfAttrs:
    mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;
}
"#;

        let expected = r#"{
  # how to string-format the option name;
  # by default one character is a short option (`-`),
  # more than one characters a long option (`--`).
  mkOptionName ? k:
    if builtins.stringLength k == 1
    then "-${k}"
    else "--${k}"
, mkOption ? k: v:
    if v == null
    then [ ]
    else [ (mkOptionName k) (lib.generators.mkValueStringDefault { } v) ]
}:
{
  toINI =
    {
      # parameter comment
      mkSectionName ? (name: libStr.escape [ "[" "]" ] name)
    , mkKeyValue ? mkKeyValueDefault { } "="
    }: attrsOfAttrs:
    mapAttrsToStringsSep "\n" mkSection attrsOfAttrs;
}
"#;

        let edits1 = vec![
            (1..4, "\n"),
            (165..168, " "),
            (170..171, "\n"),
            (359..361, "\n"),
            (368..369, "\n"),
            (370..373, "\n"),
            (486..486, " "),
        ];
        let edits2 = vec![
            (1..2, "\n  "),
            (166..167, "\n    "),
            (252..255, "\n    "),
            (267..270, "\n    "),
            (278..281, "\n    "),
            (355..356, "\n  "),
            (363..364, "\n    "),
            (365..366, "\n      "),
            (385..388, "\n      "),
            (442..445, "\n    "),
            (485..488, "\n    "),
        ];
        assert_eq!(expected, apply(&apply(text, &edits1), &edits2));

        let new_edits = merge(&edits1, &edits2);
        assert_eq!(expected, apply(text, &new_edits));
    }
}
