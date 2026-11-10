; Redox injects no other language.
;
; Plan 41 criterion 6 and decision 7: the file exists to **say so**, because a
; missing `injections.scm` reads as an oversight and a present empty one reads
; as a decision. Redox has no embedded-language construct — no inline SQL, no
; regex literal (doc 03 §11.3 declines one on the grounds that a regex is an
; ordinary crate call over a raw string), and no `quote!`-style macro, since
; Redox has no macro-call syntax at all.
;
; If one ever arrives, this is where it goes.
