# Strobe — Publication Targets and Requirements

Nice Thoughts Ranch · publication reference for *Strobe*

Requirements as documented **2026-10-20**, compiled from each platform's
official developer/partner documentation (source URLs inline). Where a
platform does not publish a term, the entry says so — "portal-gated"
means the figure or rule exists only inside the partner portal or the
signed publishing agreement. Nothing in this file is estimated.

## Route (studio decision D4, 2026-10-19)

1. **Steam** — PC launch, first.
2. **EroLabs** — the Android route, when a mobile build exists (Google
   Play prohibits adult content, so a mainstream Android store is not an
   option for this title).
3. **Nutaku** — phase two, approached with Steam results in hand.

**itch.io** is documented as a fourth section because it is the studio's
existing channel (account owned). It is **not** part of the ruled D4
route.

---

## 1. Steam (Valve / Steamworks)

**Route position:** 1 — PC launch target.

**Account prerequisites.** A Steamworks partner account is created from
a personal Steam account. Onboarding requires, in order: electronically
signing the Non-Disclosure Agreement and the Steam Distribution
Agreement; accurate legal identification of the signing entity (the
legal name must match bank and tax documents — no DBAs or aliases);
bank details for payment (account-holder name must match the legal
identification); and a tax interview — W-9-equivalent information for US
persons/entities, W-8BEN for treaty-country nationals. Submitted tax
information is verified by a third party and verification may take up
to 30 days.
([Onboarding](https://partner.steamgames.com/doc/gettingstarted/onboarding))

**Fees.** The Steam Direct Fee is **$100 USD (or equivalent) per
product**, payable with any Steam-supported payment method except
Steam Wallet funds. It is not refundable, but it is recoupable: it is
repaid in the payment made after the product reaches **$1,000 Adjusted
Gross Revenue**. VAT/GST may be charged on the fee itself depending on
country.
([Steam Direct Fee](https://partner.steamgames.com/doc/gettingstarted/appfee))

**Adult-content policy.** Adult content is distributable on Steam when
it is labeled and age-gated. The standing content rules prohibit, among
other items: nude or sexually explicit images of **real people**;
"adult content that isn't appropriately labeled and age-gated"; content
that **exploits children in any way**; content violating the laws of
any jurisdiction where it will be available; and — added to the rules in
2025 — "content that may violate the rules and standards set forth by
Steam's payment processors and related card networks and banks, or
internet network providers. In particular, certain kinds of adult only
content."
([Onboarding — Rules and Guidelines](https://partner.steamgames.com/doc/gettingstarted/onboarding))

Before review, every product must complete the **Content Survey**, in
three mandatory sections:

- **General Content** — answers generate regional ratings displayed on
  the store page (including IARC-generated ratings).
- **Mature Content** — the developer must disclose **all** adult content
  present in the build, even content that is not accessible or presented
  in the product. Titles marked "Adult Only Sexual Content" are
  compared against the build during review.
- **Generative AI Content** — AI-created content that ships with the
  game must be disclosed, split into *pre-generated* (made during
  development) and *live-generated* (made while the game runs; guardrail
  description required). Valve states it does not ship **live-generated
  Adult Only Sexual Content** at this time. Pre-generated AI artwork is
  disclosable and reviewed like other content.

Once the build and store page are approved, some survey answers cannot
be edited without contacting Steam Support — disclosures must be right
the first time.
([Content Survey](https://partner.steamgames.com/doc/gettingstarted/contentsurvey))

**Regional restrictions.** The public documentation establishes the
duties rather than a country list: the product must comply with the
laws of every jurisdiction where it is available; the General Content
survey produces region-specific ratings; and availability is configured
per territory in Steamworks. Whether an Adult Only title may be sold
in a given territory (for example, jurisdictions whose age-verification
law Steam does not implement) is confirmed in the partner portal at
submission time — **not stated in the public documentation**.

**Technical requirements.** Builds are uploaded with **SteamPipe**
(content is chunked, compressed, and encrypted; depots per
OS/language; public and private beta branches; a dedicated build
account with "Edit App Metadata" and "Publish App Changes To Steam"
permissions is recommended).
([Uploading to Steam](https://partner.steamgames.com/doc/sdk/uploading))
Steamworks features (achievements, cloud saves, trading cards, etc.)
are opt-in, but review enforces honesty: **all features listed on the
store page must be implemented in the submitted build**, the product
must launch on every OS the store page claims, and **any in-game
transactions must use Steam Wallet** — the product may not link to
store pages that do not offer Steam Wallet.
([Review Process](https://partner.steamgames.com/doc/store/review_process))

**Review and timing.** Store presence and product build are each
reviewed by Valve: typically **3–5 business days** each, and Valve asks
developers to allow at least 7 business days. For Adult Only titles,
**both** the store page and the build must be complete and submitted
before either is reviewed, and adult reviews "can take longer than the
typical 3–5 business days." Two onboarding clocks also apply before a
first release: a **21-day waiting period** between paying the app fee
and release, and the store page must be publicly visible as "Coming
Soon" for **at least two weeks** before launch. Approval is one-time —
after release, updates do not require re-review.
([Review Process](https://partner.steamgames.com/doc/store/review_process);
[Onboarding](https://partner.steamgames.com/doc/gettingstarted/onboarding))

**Revenue share and payouts.** Valve's published tier structure
(announced 2018-11-30, applied to revenue from 2018-10-01): Valve takes
**30%** of Adjusted Gross Revenue up to $10M per title, **25%** on the
portion between $10M and $50M, and **20%** above $50M; the accrual
includes game packages, DLC, in-game sales, and Community Market fees.
The binding terms are those of the Steam Distribution Agreement signed
at onboarding.
([Valve announcement as published](https://www.gameinformer.com/2018/11/30/valve-adjusting-revenue-share-for-steams-most-popular-games?amp);
[Reporting and Payments FAQ](https://partner.steamgames.com/doc/finance/payments_salesreporting/faq))
Payouts are monthly — **paid by the 30th of the month following
sales** — by EFT only (ACH within the US, USD SWIFT wire elsewhere; USD
only). Payments may be held until a **$100 minimum payout** is earned,
and partners can set a higher hold threshold. US partners are reported
on Form 1099-MISC; non-US partners on Form 1042-S, with withholding per
the tax interview.
([Reporting and Payments FAQ](https://partner.steamgames.com/doc/finance/payments_salesreporting/faq))

---

## 2. EroLabs

**Route position:** 2 — the Android route, entered when a mobile build
exists.

**Platform model.** EroLabs is an adults-only game platform: its
catalog is explicit adult titles, distributed through EroLabs' own
website and platform app rather than through Google Play (which
prohibits adult content). Third-party titles are carried under
partnership/business-development arrangements; EroLabs does not operate
a public self-serve developer portal comparable to Steam Direct.
([EroLabs official site](https://www.erolabs.com/))

**Account prerequisites.** By partnership agreement; no public
self-serve signup, identity, tax, or banking requirements are
published. **Portal/partnership-gated.**

**Fees.** No public fee schedule (no published per-title fee or listing
fee). **Not public — set in the partnership agreement.**

**Adult-content policy.** Explicit adult content is the platform's
core catalog, behind an 18+ gate. The categories of content EroLabs
prohibits, and its content-review criteria, are **not published** —
they are set in the partnership agreement. **Partnership-gated.**

**Technical requirements.** Titles ship as Android builds distributed
through the EroLabs platform (direct distribution outside Google Play)
and, for some titles, browser-playable versions. Any SDK integration
(account, payment) is specified in the partnership documentation.
**Not public — partnership-gated.**

**Review and timelines.** Curated onboarding by agreement; no public
review process or timeline. **Partnership-gated.**

**Revenue share and payouts.** No public rate or payout terms.
Third-party business-development pages claim developer-favorable splits,
but EroLabs publishes no figure itself — treat any number as
**unverified** until it appears in an agreement.

---

## 3. Nutaku

**Route position:** 3 — phase two, approached with Steam results in
hand.

**Platform model and account prerequisites.** Nutaku is an adults-only
(18+) game platform (operated by Aylo). It is **not** a developer and
not a self-serve store: in its own words, "We partner with talented
developers from around the world to bring their Adult and Ecchi games
to a wide audience," spanning "free-to-play browser games to premium
downloadable experiences." Onboarding is therefore by partnership —
a signed publishing/distribution agreement — not an open signup.
Identity, tax, and banking steps are part of that agreement and are
**not public / portal-gated**.
([Nutaku support — "Is Nutaku a game developer?"](https://nutaku.freshdesk.com/support/solutions/articles/43000647091-is-nutaku-a-game-developer-))

**Fees.** No public per-title or listing fee. **Not public —
agreement-gated.**

**Adult-content policy.** Explicit adult content is the platform's
purpose: the .net storefront is the adults-only property, and the
catalog is uncensored adult titles across browser, downloadable PC, and
Android. The specific content standards a submitted title must meet
(prohibited categories, depiction rules) live in the publishing
agreement and partner onboarding materials — **not public /
portal-gated**.

**Technical requirements.** This is the one area Nutaku documents
publicly, via its official technical documentation. Games integrate the
**Nutaku SDK** (the current documentation covers SDK 3.x), published in
Native (Android), Unity, and Web variants, backed by the **Nutaku Game
Platform REST API**:

- **People API** — current/other player profiles.
- **Payment API** — in-game purchases: the game creates a payment, the
  platform presents its transaction page, and purchases settle in
  platform currency.
- **makeRequest API** — OAuth RSA-signed calls from the game to the
  developer's own server through Nutaku's servers.

Supported distribution shapes per the same documentation: browser
games, downloadable games, and Android games delivered through the
**Nutaku Android Store client app** (cross-game deep-linking opens the
client at a title's page, or the website when the client is absent).
Each title is assigned an App ID and a title-id.
([Nutaku Game Platform REST API (Native SDK)](https://nutakudoc.atlassian.net/wiki/spaces/ND/pages/27754525/Nutaku+Game+Platform+REST+API+Native+SDK);
[Deep-linking to a different Nutaku game](https://nutakudoc.atlassian.net/wiki/spaces/ND/pages/2980347905/Deep-linking+to+a+different+Nutaku+game))

**Review and timelines.** Titles are reviewed and onboarded under the
partnership process; no public review criteria or timelines.
**Portal-gated.**

**Revenue share and payouts.** No public rate. Nutaku does not publish
its split, payout cadence, thresholds, or methods; all are set in the
publishing agreement. **Not public — agreement-gated** (any figure from
third-party articles is unverified).

---

## 4. itch.io — existing channel (not part of the D4 route)

**Status.** The studio's itch.io account is owned and active
(**NTRanch**, display name "Nice Thoughts Ranch"). This section
documents the channel as it stands; it is not a ruled publication
target for *Strobe*.

**Account prerequisites.** A standard itch.io account; no developer
application. To be **paid**, a tax interview is required: with the
"Collected by itch.io" payout mode the interview is completed once
with itch.io and the information must reach **Validated** state during
payout review before any payment is sent; with "Direct to you" mode,
tax identity is given to each payment provider (PayPal/Stripe)
directly.
([Accepting Payments and Getting Paid](https://itch.io/docs/creators/payments))

**Fees.** No listing or upload fee — "itch.io costs nothing to use."
The platform's cut is **open revenue sharing**: the seller chooses
itch.io's share from **0% to 100%**; the default is **10%**. Payment
processor fees are separate and per transaction (docs state "for most
transactions" **$0.30 + 2.9%** for both PayPal and Stripe).
([Creator FAQ](https://itch.io/docs/creators/faq);
[Payments](https://itch.io/docs/creators/payments))

**Adult-content policy.** "Fictional, illustrated, and rendered content
is generally fine, assuming it's legal." Hard prohibitions in itch.io's
own text: **no sexualized images or videos of real-life humans**; **no
AI-generated imagery designed to resemble photographic content of real
people**; **no content glorifying sexual violence**; and "depictions of
minors, minor-presenting, or suggested minors in a sexual context are
not allowed and will result in account suspension." Violations are
permanent with no appeal, and funds on a violating account are not
eligible for payout.

Separately, **paid** content must also satisfy the acceptable-use
policies of the payment processors the account uses. itch.io publishes
the card networks' non-exhaustive prohibited-theme list for works sold
on the platform: non-consensual content (real or implied); underage or
"barely legal" themes; incest or pseudo-incest; bestiality; rape,
coercion, or force-related content; sex-trafficking implications;
revenge porn / voyeur / hidden cam; and fetish content involving bodily
waste or extreme harm. Works containing those themes cannot be sold
through itch.io.
([Creator FAQ — "Is adult content allowed?"](https://itch.io/docs/creators/faq))

**Labeling.** Adult projects **must be marked** via Metadata »
Classification on the project edit page; adult content is only shown
to accounts that have enabled it, and **explicit images may not be used
as the cover image**. Failure to classify can remove all of the
account's pages from discovery indexing or permanently disable payments
on the account.
([Quality Guidelines — "Adult content must be labeled"](https://itch.io/docs/creators/quality-guidelines))

**Technical requirements.** Files are hosted and delivered **exactly
as uploaded** — itch.io applies no DRM and makes no modifications — so
any platform executable can be shipped. New accounts carry soft limits
(20 project pages; per-page file count/size limits) that itch.io raises
on request.
([Creator FAQ](https://itch.io/docs/creators/faq))

**Review and indexing.** There is no pre-publication approval: pages go
live when the creator publishes them. Discoverability is governed by
the quality guidelines — pages that follow them are eligible for
indexing in browse/search/recommendations; moderators or automated
systems may de-index pages that do not (the page stays reachable by
direct URL). Payouts, by contrast, are staff-reviewed (below).
([Quality Guidelines](https://itch.io/docs/creators/quality-guidelines))

**Payouts.** Two modes. **Direct to you**: each sale settles straight
to the seller's PayPal/Stripe; the seller is merchant of record and is
responsible for VAT. **Collected by itch.io (Payouts)**: itch.io is
merchant of record, collects and remits VAT, and pays out via PayPal
or Payoneer on request. Payout eligibility: tax interview completed,
at least **$5.00 USD** balance, revenue becomes available **7 days**
after purchase, and staff review every payout — a minimum of 7 days,
typically **10–14 days**. The collected mode is USD-only.
([Payments](https://itch.io/docs/creators/payments))

---

## Common to all targets

Drawn from the platform sections above — the requirements *Strobe*
must satisfy no matter which storefront it ships on:

1. **A documented adult (18+) cast.** itch.io bans depictions of
   minors — including minor-presenting or merely "suggested" minors —
   in a sexual context, with account suspension and forfeited payouts;
   Steam bans content that exploits children in any way. Every other
   target is an adults-only platform whose agreements presume an adult
   cast. The studio's cast documentation (all ten characters depicted
   18+) is the evidence base on every route and must be kept current.
2. **Complete, truthful content disclosure.** Steam's Content Survey
   must disclose *all* adult content in the build — even content not
   reachable in play — plus generative-AI use (split pre-generated vs.
   live-generated; live-generated adult sexual content is not shippable
   on Steam at all). itch.io requires the adult classification to be
   set, with de-indexing or payment disablement for mislabeling. Assume
   the same honesty duty, in agreement form, on the partnership routes.
3. **Descriptors and labeling before marketing.** Store art is
   reviewed too: Steam reviews capsule images and screenshots (gameplay
   only), itch.io forbids explicit cover images, and both platforms
   gate adult pages behind account-level age/opt-in controls. *Strobe*'s
   descriptors must be declared truthfully at first submission — Steam
   locks parts of the survey after approval.
4. **Payment-processor content limits apply everywhere.** Steam's rule
   15 subordinates content to card-network and bank standards
   ("certain kinds of adult only content"), and itch.io publishes the
   processors' prohibited-theme list outright. That list is the
   practical floor for every target: no non-consensual (real or
   implied), incest or pseudo-incest, bestiality, rape/coercion/force,
   sex-trafficking, voyeur/hidden-cam, or underage/"barely legal"
   themes, and nothing sexualizing real people.
5. **Identity, tax, and banking readiness precede revenue.** Steam
   requires verified legal identity, matching bank details, and a
   completed tax interview before onboarding completes; itch.io
   requires a completed, validated tax interview before the first
   payout. The partnership platforms fold the equivalent steps into
   their agreements — none of them is a step that can be improvised at
   launch.
6. **Regional law is the developer's problem.** Steam requires the
   product to comply with the laws of *every* jurisdiction where it is
   available, generates regional ratings from the survey, and leaves
   territory configuration to the developer. *Strobe* must be withheld
   from territories where its sale would be unlawful, per platform.

---

*Compiled by Pax for Nice Thoughts Ranch. Verify figures against the
cited sources before acting on them — platform terms change, and the
portal-gated items can only be confirmed inside each platform's partner
portal or signed agreement.*
