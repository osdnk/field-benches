# Cross-Field LIOP for a Binius-Style Boolean Witness

## Goal

We have a Binius-style Boolean witness packed over

\[
F^{(0)}=\mathbb F_{2^{128}},
\]

but the available PCS operates over

\[
F^{(1)}=\mathbb F_{2^{162}}.
\]

There is no field embedding \(F^{(0)}\hookrightarrow F^{(1)}\). The goal is therefore to transform a final linear/MLE claim over the \(F^{(0)}\)-packed witness into a claim about an \(F^{(1)}\)-packed witness, using only the common base field

\[
B=\mathbb F_2.
\]

The construction is inspired by Construction 3.1 (ring switching), but uses a direct basis decomposition followed by one random batching and one sumcheck.

---

## Setup

Let

\[
w:\mathbb B^7\times\mathbb B^\ell\to B
\]

be the Boolean witness. Here \(i\in\mathbb B^7\) indexes one of the 128 bits packed into a field element and \(y\in\mathbb B^\ell\) indexes packed witness positions.

Fix a \(B\)-basis

\[
(\beta_i)_{i\in\mathbb B^7}
\]

of \(F^{(0)}\).

Define the \(F^{(0)}\)-packing

\[
\pi^{(0)}(y)
=
\sum_{i\in\mathbb B^7}w(i,y)\beta_i
\in F^{(0)}.
\]

Choose 128 \(B\)-linearly independent elements

\[
(\gamma_i)_{i\in\mathbb B^7}\subset F^{(1)}
\]

and define

\[
\pi^{(1)}(y)
=
\sum_{i\in\mathbb B^7}w(i,y)\gamma_i
\in F^{(1)}.
\]

The actual PCS commitment is to the MLE of \(\pi^{(1)}\).

Define the \(B\)-linear injection

\[
\phi:F^{(0)}\to F^{(1)}
\]

by

\[
\phi(\beta_i)=\gamma_i.
\]

Thus, for every packed witness value,

\[
\phi(\pi^{(0)}(y))=\pi^{(1)}(y).
\]

Importantly, \(\phi\) is only \(B\)-linear; it is not a field homomorphism.

---

## Input claim

Assume the preceding Binius-style LIOP has reduced the statement to an MLE claim

\[
\widetilde{\pi}^{(0)}(r)=s,
\qquad
r\in(F^{(0)})^\ell,
\quad
s\in F^{(0)}.
\]

Equivalently,

\[
s
=
\sum_{y\in\mathbb B^\ell}
\operatorname{eq}(r,y)\pi^{(0)}(y).
\tag{1}
\]

The purpose of the switch LIOP is to reduce (1) to an opening of \(\widetilde{\pi}^{(1)}\).

---

## Step 1: Decompose the equality coefficients

For every \(y\in\mathbb B^\ell\), decompose

\[
\operatorname{eq}(r,y)\in F^{(0)}
\]

in the basis \((\beta_k)\):

\[
\operatorname{eq}(r,y)
=
\sum_{k\in\mathbb B^7}
\operatorname{eq}'(r,y,k)\beta_k,
\]

where

\[
\operatorname{eq}'(r,y,k)\in B.
\]

Define

\[
s_k
:=
\sum_{y\in\mathbb B^\ell}
\operatorname{eq}'(r,y,k)\pi^{(0)}(y)
\in F^{(0)}.
\tag{2}
\]

Then (1) implies

\[
\boxed{
s=\sum_{k\in\mathbb B^7}\beta_k s_k.
}
\tag{3}
\]

The prover sends the 128 values \(s_k\), and the verifier checks (3).

This check alone does **not** certify the individual \(s_k\)'s; their correctness is established by the next random batching step.

---

## Step 2: Flip to \(F^{(1)}\)

The verifier computes

\[
s'_k:=\phi(s_k)\in F^{(1)}.
\]

Since \(\operatorname{eq}'(r,y,k)\in B\) and \(\phi\) is \(B\)-linear,

\[
\begin{aligned}
s'_k
&=
\phi\left(
\sum_y
\operatorname{eq}'(r,y,k)\pi^{(0)}(y)
\right)\\
&=
\sum_y
\operatorname{eq}'(r,y,k)\pi^{(1)}(y).
\end{aligned}
\]

Thus the desired consistency relations are

\[
\boxed{
s'_k
=
\sum_{y\in\mathbb B^\ell}
\operatorname{eq}'(r,y,k)\pi^{(1)}(y)
\qquad
\forall k\in\mathbb B^7.
}
\tag{4}
\]

---

## Step 3: Randomly batch the 128 relations

After the prover has sent the \(s_k\)'s, the verifier samples

\[
r'\gets(F^{(1)})^7.
\]

Define

\[
s'
:=
\sum_{k\in\mathbb B^7}
\operatorname{eq}(r',k)s'_k.
\]

Taking the same random linear combination of (4) gives

\[
\boxed{
s'
=
\sum_{y\in\mathbb B^\ell}
\left(
\sum_{k\in\mathbb B^7}
\operatorname{eq}(r',k)
\operatorname{eq}'(r,y,k)
\right)
\pi^{(1)}(y).
}
\tag{5}
\]

Crucially, the \(k\)-coordinate has already been batched. Therefore the subsequent sumcheck runs **only over \(y\)** and has \(\ell\) rounds, not \(\ell+7\) or \(\ell+14\).

---

## Step 4: One sumcheck

Run an \(\ell\)-round sumcheck on the right-hand side of (5).

Let

\[
r''\gets(F^{(1)})^\ell
\]

be the final sumcheck point. The final sumcheck check has the form

\[
t
=
\left(
\sum_{k\in\mathbb B^7}
\operatorname{eq}(r',k)
\widetilde{\operatorname{eq}'}(r,r'',k)
\right)
\widetilde{\pi}^{(1)}(r''),
\tag{6}
\]

where \(\widetilde{\operatorname{eq}'}\) denotes the multilinear extension, in the \(y\)-variable, of the Boolean function

\[
y\mapsto\operatorname{eq}'(r,y,k).
\]

Everything in (6) except

\[
\widetilde{\pi}^{(1)}(r'')
\]

is verifier-known.

Therefore the switch reduces to one ordinary PCS opening claim

\[
\boxed{
\widetilde{\pi}^{(1)}(r'')=z.
}
\]

The verifier invokes the existing \(F^{(1)}\)-PCS on this claim.

---

## Soundness of the batching step

Suppose some relation in (4) is false. Define

\[
e_k
=
s'_k-
\sum_y
\operatorname{eq}'(r,y,k)\pi^{(1)}(y).
\]

Then \(e=(e_k)_{k\in\mathbb B^7}\neq0\).

Consider its MLE

\[
E(R)
=
\sum_{k\in\mathbb B^7}
\operatorname{eq}(R,k)e_k.
\]

Since

\[
E(k)=e_k
\]

on the Boolean cube, \(E\) is nonzero whenever some \(e_k\neq0\). It is multilinear in seven variables, hence

\[
\deg E\le7.
\]

Therefore a fresh

\[
r'\gets(F^{(1)})^7
\]

detects an incorrect vector except with probability at most

\[
\frac{7}{|F^{(1)}|}.
\]

The challenge \(r'\) must be sampled only after the prover has committed to the values \(s_k\).

---

## Communication and round complexity

The switch requires:

- 128 elements \(s_k\in F^{(0)}\);
- one fresh \(7\)-dimensional batching challenge \(r'\);
- one \(\ell\)-round sumcheck over \(F^{(1)}\);
- one final \(F^{(1)}\)-PCS opening.

In particular, there is **no additional 7-round sumcheck over the basis coordinate**. The basis coordinate is eliminated by random batching before the sumcheck.

---

## End-to-end role in the AND protocol

The preceding Binius-style Boolean/BitAnd LIOP may operate naturally over

\[
F^{(0)}=\mathbb F_{2^{128}}
\]

and eventually produce a claim

\[
\widetilde{\pi}^{(0)}(r)=s.
\]

The protocol above then performs

\[
\boxed{
F^{(0)}\text{-MLE claim}
\longrightarrow
B\text{-coordinate decomposition}
\longrightarrow
F^{(1)}\text{-linear query}
\longrightarrow
F^{(1)}\text{-PCS opening}.
}
\]

Thus the Boolean/AND machinery can retain the convenient \(128\)-bit packing, while the actual polynomial commitment scheme operates over

\[
F^{(1)}=\mathbb F_{2^{162}}.
\]

No field embedding

\[
F^{(0)}\hookrightarrow F^{(1)}
\]

is required.

---

## Remaining technical point

For a concrete implementation, one must give an efficient procedure for evaluating

\[
\widetilde{\operatorname{eq}'}(r,r'',k)
\]

at the final sumcheck point. The protocol above is algebraically correct assuming this verifier-known coefficient function can be evaluated efficiently. A paper treatment should either provide that evaluation algorithm explicitly or retain the relevant coordinate as a sumcheck variable if doing so is cheaper.