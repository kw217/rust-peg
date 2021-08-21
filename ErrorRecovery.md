peg-runtime: error.rs

ErrorState is a whole lot of stuff: furthest error pos, suppressing?, reparsing?, current expected set.

peg-runtime: lib.rs

RuleResult<T> is either Matched(usize, T) or Failed.

peg-macros: translate.rs

ParseState contains a cache of rule parses (maps from usize to RuleResult).

compile_rule compiles a rule (for internal use) into a function returning a RuleResult (and maintaining mutable ParseState and ErrorState).

compile_rule_export compiles a rule (for toplevel entry) into a function returning a `Result<T, ParseError<usize>>`.
It parses the input, then checks the end. If EOF, it returns OK. If not end, it marks as a failure, then reparses in error mode; when it fails again it returns a detailed error. The second pass is there to pick up the expected set.
A parse error is just a location and an expected set.

We want to return multiple errors, so the "export" function return type
needs to be (T, Vec<Err>) or similar. (not sure if Vec is best).

RuleResult should keep Vec<Err> around. Can it fail?

Paper says rule returns (Matched(usize, T) | Failed | String), Option<usize - furthest>, Vec<(String err, usize where)>.

Need new boolean "recovery allowed" (i.e., per the paper Figure 4, is R preserved or []).

Recovered errors is cumulative, so can be part of the ErrorState (i.e., mutable rather than passed around).

Furthest failure offset sadly is *not* cumulative; a label trumps any other possible failures, and it combines in interesting ways. But the parser works linearly so we could get away with it there.

Furthest failure offset is valid whether result is success, failure, or label.

lpeglabel calls the three states (success), ordinary failure, error. So let's call label "error".

RuleResult<T> {
    Matched(usize, T)
    Failed
    Error(String)
}

and pair that with an Option<usize> for furthest failure offset.

Put a Vec<(String, usize)> into the ErrorState.

String in the above is a label, so has associated recovery info.

Does this mean actual failures are not recovered? Maybe. Yes.

Does Error need T? Not sure - is it just Throw that handles recovery, or is it labelled failures too?

Beef up the tests or ensure they are well covered. >> looks OK to me; only one test that looks at error positions but otherwise seems to have good coverage. Hopefully that's enough.



See https://github.com/sqmedeiros/lpeglabel and https://arxiv.org/abs/1806.11150

