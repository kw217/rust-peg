peg-runtime: error.rs

ErrorState is a whole lot of stuff: furthest failure pos, suppressing?, reparsing?, current expected set.

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


# Thoughts on the paper

Notice that:

* If the parse succeeds, it may or may not have a furthest failure. Furthest failure in this case can only be introduced by rep.1, indicating how far we got attempting to parse one more repetition.
* If the parse fails, it may or may not have a furthest failure. It can only lose its furthest failure from not.2.
* If the parse throws, it must always have a furthest failure.
* There are just two rules - not.1 and not.2 - which do not propagate the furthest failure through in the obvious fashion.

I think not.2 should set the furthest failure to the current location, rather than nothing at all. Parsing b with !b should yield "unexpected mismatch at column 1", not "unexpected".

If the recovery expression fails or throws (throw.3), it seems appropriate to ignore the recovery expression and throw the original label. In particular, allowing it to fail could permit subsequent backtracking.

Sérgio Medeiros in private communication suggests varying throw.2 and throw.3 to disable nested recovery expressions. This avoids cycles, at the cost of some power. We adopt this suggestion.

I think seq.3, rep.4, and ord.5 (the only rules which combine a throw with something else) should all be consistent. They should set the furthest failure to the error location; a prior or alternate furthest failure is not very interesting. In other words, rep.4 should return `z` as the furthest failure.

Errors are more interesting than failures, because they indicate a definite violation of an expectation.
They always propagate immediately, unlike failures where backtracking is possible and we're only interested in which failure happened the furthest into the input.
No such furthest-location logic is needed for errors.


