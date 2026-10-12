# Frozen marketplace data for the tests

A copy of `vendor/marketplace/` at upstream commit
b1d0ce14316efbb3e2310a022c40ef394f8dda50, from before the marketplace
removed CHAP-EWARS. The unit tests embed it in place of the vendored copy,
and the CLI tests start with it in their registry cache. So a change in the
marketplace does not break a test.

Do not refresh it with `scripts/vendor-marketplace.sh`. Change it only on
purpose, with the reason in the commit.
