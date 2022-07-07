# `severity` and `user_base_impact`

To inform our prioritization, we want to track how severe an issue is and how many users are affected. For each case, this is a judgment call by the person adding or editing the entry. If an issue has multiple site breakage reports, we should consider the highest severity and the highest reach when adding the value to an entry.

## Severity

The severity can be one of four values:

- `critical`: The site is breaking in a way that forces users to use another browser.
- `high`: The issue can be worked around, but it's annoying enough that a user might feel compelled to use another browser.
- `normal`: The issue is real but can be worked around by the user inside Firefox.
- `low`: Minor visual/UX issues that do not prevent the users from getting their tasks done.

## Impact

In the `user_base_impact` we keep track of the estimated reach of an issue:

- `large`: affecting very popular sites.
- `medium`: affecting some slightly popular sites, or a larger number of small sites.
- `small`: affecting only small sites.
- `unknown`: it isn't clear what is the number of affected users at the moment.
