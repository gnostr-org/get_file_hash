// n34 - A CLI to interact with NIP-34 and other stuff related to codes in nostr
// Copyright (C) 2025 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://gnu.org/licenses/gpl-3.0.html>.

use super::*;

#[test]
fn patch_normal() {
    let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: a to abc

Abc patch
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

- a
+ abc
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(patch.subject, "[PATCH] chore: a to abc");
    assert_eq!(patch.body, "Abc patch");
}

#[test]
fn patch_normal_with_patch_in_content() {
    let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: Subject in subject

A good test patch
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: What a subject

hi
---
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(patch.subject, "[PATCH] chore: Subject in subject");
    assert_eq!(patch.body, "A good test patch");
}

#[test]
fn patch_multiline_subject() {
    let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: Some long subject yes so long one Some long subject yes
 so long one

Abc patch
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

- a
+ abc
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(
        patch.subject,
        "[PATCH] chore: Some long subject yes so long one Some long subject yes so long one"
    );
    assert_eq!(patch.body, "Abc patch");
}

#[test]
fn patch_multiline_body() {
    let patch_content = r#"From 24e8522268ad675996fc3b35209ce23951236bdc Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 27 May 2025 19:20:42 +0000
Subject: [PATCH] chore: a to abc

Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur.
---
 src/nostr_utils/mod.rs            |  1 +
 1files changed, 3 insertions(+), 1 deletions(-)

diff --git a/src/nostr_utils/mod.rs b/src/nostr_utils/mod.rs
index 4120f5a..e68783c 100644
--- a/src/nostr_utils/mod.rs
+++ b/src/nostr_utils/mod.rs
@@ -103,31 +103,9 @@ impl CommandRunner for NewArgs {

- a
+ abc
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(patch.subject, "[PATCH] chore: a to abc");
    assert_eq!(
        patch.body,
        "Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur."
    );
}

#[test]
fn patch_cover_letter() {
    let patch_content = r#"From 864f3018f62ab2e1265edb670d5493dafe7d2cb2 Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 3 Jun 2025 08:41:12 +0000
Subject: [PATCH v2 0/7] feat: Some test just a test

Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(patch.subject, "[PATCH v2 0/7] feat: Some test just a test");
    assert_eq!(
        patch.body,
        "Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a"
    );
}

#[test]
fn patch_multiline_cover_subject() {
    let patch_content = r#"From 864f3018f62ab2e1265edb670d5493dafe7d2cb2 Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 3 Jun 2025 08:41:12 +0000
Subject: [PATCH v2 0/7] feat: Some test just a test some test just a test some
 test just a test

Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(
        patch.subject,
        "[PATCH v2 0/7] feat: Some test just a test some test just a test some test just a test"
    );
    assert_eq!(
        patch.body,
        "Cover body

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a"
    );
}

#[test]
fn patch_multiline_cover_body() {
    let patch_content = r#"From 864f3018f62ab2e1265edb670d5493dafe7d2cb2 Mon Sep 17 00:00:00 2001
From: Awiteb <a@4rs.nl>
Date: Tue, 3 Jun 2025 08:41:12 +0000
Subject: [PATCH v2 0/7] feat: Some test just a test some test just a test some
 test just a test

Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur.

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a
-- 
2.49.0"#;
    let patch = GitPatch::from_str(patch_content).unwrap();
    assert_eq!(
        patch.subject,
        "[PATCH v2 0/7] feat: Some test just a test some test just a test some test just a test"
    );
    assert_eq!(
        patch.body,
        "Lorem ipsum dolor sit amet. 33 laborum galisum aut fugiat dicta vel accusamus
aliquam vel quisquam fuga in incidunt voluptas a aliquid neque ab iure pariatur.
Et molestiae vero a consectetur laborum et accusantium sequi. Et ratione
atque et molestiae dolorem in asperiores amet id dolor corporis in adipisci
aspernatur.

Awiteb (1):
  chore: Update `README.md`

 README.md      |  2 +-


base-commit: f670859b92d525874fd621452080c8479964ac6a"
    );
}

#[test]
fn normal_patch_filename() {
    let mut patch = GitPatch {
        inner:   String::new(),
        subject: String::new(),
        body:    String::new(),
    };

    patch.subject = "[PATCH v2 0/3] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("v2-0000-cover-letter.patch")
    );
    patch.subject = "[PATCH 0/3] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("0000-cover-letter.patch")
    );
    patch.subject = "[PATCH v2 1/3] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("v2-0001-feat-some-test-just-a-test.patch")
    );
    patch.subject = "[PATCH v42 1/3] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("v42-0001-feat-some-test-just-a-test.patch")
    );
    patch.subject = "[PATCH v42 23/30] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("v42-0023-feat-some-test-just-a-test.patch")
    );
    patch.subject = "[PATCH 1/3] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("0001-feat-some-test-just-a-test.patch")
    );
    patch.subject = "[PATCH 32/50] feat: Some test just a test".to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("0032-feat-some-test-just-a-test.patch")
    );
    patch.subject = "[PATCH v100 32/50] feat: some long subject some long subject some long \
                     subject some long subject"
        .to_owned();
    assert_eq!(
        patch.filename("").unwrap(),
        PathBuf::from("v100-0032-feat-some-long-subject-some-long-subject-some-long-subject.patch")
    );
}

#[test]
fn patch_filename_without_patch() {
    let mut patch = GitPatch {
        inner:   String::new(),
        subject: "[RFC v5 1/2] Something".to_owned(),
        body:    String::new(),
    };

    assert!(patch.filename("").is_err());
    patch.subject = "Something".to_owned();
    assert!(patch.filename("").is_err());
}

#[test]
fn patch_filename_without_number() {
    let mut patch = GitPatch {
        inner:   String::new(),
        subject: "[PATCH v5 /2] Something".to_owned(),
        body:    String::new(),
    };

    assert!(patch.filename("").is_err());
    patch.subject = "[PATCH v5 2/] Something".to_owned();
    assert!(patch.filename("").is_err());
}

#[test]
fn patch_filename_without_version() {
    let patch = GitPatch {
        inner:   String::new(),
        subject: "[PATCH 1/2] Something".to_owned(),
        body:    String::new(),
    };

    assert!(patch.filename("").is_ok());
}
