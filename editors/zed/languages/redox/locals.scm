; Scopes and bindings, for an editor's rename and highlight-occurrences.
;
; Deliberately shallow, for §1.2's reason: resolving a name to its definition
; is what plan 40 §6 declined as needing a symbol table Redox does not have.
; What is here is the structural part — a block is a scope, and the places a
; name is *introduced* are known from the grammar without resolving anything.

; ------------------------------------------------------------------ scopes

(source_file) @local.scope
(block) @local.scope
(statement_block) @local.scope
(equation) @local.scope
(lambda) @local.scope
(case_arm) @local.scope
(where_clause) @local.scope
(module) @local.scope

; ---------------------------------------------------------------- bindings

(equation name: (identifier) @local.definition)
(equation parameters: (identifier) @local.definition)
(binding pattern: (pattern_run (identifier) @local.definition))
(generator binder: (identifier) @local.definition)
(for_statement binder: (identifier) @local.definition)
(draw_statement name: (identifier) @local.definition)
(lambda parameters: (identifier) @local.definition)
(import alias: (identifier) @local.definition)
(import alias: (type_identifier) @local.definition)

; ------------------------------------------------------------------- uses

(identifier) @local.reference
