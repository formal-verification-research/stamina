# Input Format

This document describes the CRN/VASS input format.

## CRN Semantics
Let a CRN be defined as a tuple $\mathcal{M} = \langle \mathfrak{X}, \mathfrak{R}, s_0 \rangle$ such that:
- $\mathfrak{R} = \{ R_1, R_2, \ldots, R_n \}$ is the set of $n$ reactions
- $\mathfrak{X} = \{ X_1, X_2, \ldots, X_m \}$ is a set of $m$ chemical species
- $s_0$ is the initial state

### Species

A species $X_\alpha$ simply represents the *name* of a species in the CRN.

In the CRN input format, declare species `X1` on its own line as follows:

```txt
species X1
```

### States

The *count* of a species is evaluated at a particular state. That is, states in a CRN are functions that map a vector of species names to a vector of non-negative-integer values (i.e., species counts), formally defined as follows:

Let $s_i: \vec{\mathfrak{X}} \to \mathbb{Z}_{\geq 0}^m$ be a function that maps each species $X_i \in \mathfrak{X}$ to its corresponding count at state $i$:
$$s_i([X_1, X_2, \ldots, X_m]) = [c_1, c_2, \ldots, c_m]$$
where $c_i \in \mathbb{Z}_{\geq 0}$ represents the count of species $X_i$ in the state $s_i$.

Programatically, this is often represented by imposing an order $<$ on $\mathfrak{X}$, then storing states as vectors of natural numbers such that the order of species is preserved from state to state.

Denote by $s_0$ the model's initial state. To describe this state for each species, append an `init` value to each species' declaration. For example, to set the initial state value of `X1` to `100`, adapt the species declaration as follows:

```txt
species X1 init 100
```

A complete model containing $m$ species should appear similar to the following:

```txt
species X1 init 100
species X2 init 200
        ...
species Xm init 398
```

In mathematical notation, this matches the description $s_i([X_1, X_2, \ldots, X_m]) = [100, 200, \ldots, 398]$

### Reactions
A reaction is a tuple $R_j = \langle C_j, P_j, \gamma_j \rangle$, where $C_j$ is the consumption vector, $P_j$ is the production vector, and $\gamma_j$ is the constant reaction rate coefficient.

$C_j$ and $P_j$ each map a species $X_i \in \mathfrak{X}$ to the count by which that species should decrease or increase (respectively) after reaction $R_j$ has executed. Programatically, this is often represented by imposing an order $<$ on $\mathfrak{X}$, then storing $C_j$ and $P_j$ as vectors of natural numbers such that the order of species is preserved between all states and reactions.

The net change after reaction $R_j$ executes at state $s_k$ is given by $s_{k+1} = s_k + P_j - C_j$. That is, the final effect after a reaction is found by adding to the produced species and subtracting from the consumed species.

$C_j$ and $P_j$ are separated because $C_j$ acts as a guard on the enabled status of a reaction. It is impossible to have negative species counts, so  a reaction is not enabled if any element of $s_k - C_j$ is negative. Formally, the guard for reaction $R_j$ at state $s_k$ is described as follows:

$$
\text{enabled}(R_j, s_k) = \forall \; 0 < i \leq m \; \cdot \; s_k[i] \geq C_j[i]
$$

A reaction can include a reaction rate constant, $\gamma_j$, which is described in-depth in related literature.

A reaction is defined over several lines. The first line declares the name of the reaction:
```txt
Reaction R1
```

Following this declaration, all tab- or space-indented lines are assumed to belong to `R1`. These lines declare $C_j$, $P_j$, and $\gamma_j$. Order of these lines does not matter.

Up to one `consume` statement per species in $\mathfrak{X}$ is permitted. These statements indicate the name of the species followed by its value in $C_j$, as follows:

```txt
  consume X1 1
  consume X2 3
      ...
  consume Xm 2
```

This creates the vector $C_j([X_1, X_2, \ldots, X_m]) = [1, 3, \ldots, 2]$.

Any species without a corresponding `consume` statement is assumed to correspond to a value of $0$ in $C_j$; that is, there is no need to include a `consume` statement if a species is not consumed. Further, if the number consumed is `1`, it is allowable to omit the number consumed.

Similarly, up to one `produce` statement per species in $\mathfrak{X}$ is permitted. These statements indicate the name of the species followed by its value in $P_j$, as follows:

```txt
  produce X1 1
  produce X2 3
      ...
  produce Xm 2
```

This creates the vector $P_j([X_1, X_2, \ldots, X_m]) = [1, 3, \ldots, 2]$.

Any species without a corresponding `produce` statement is assumed to correspond to a value of $0$ in $P_j$; that is, there is no need to include a `produce` statement if a species is not consumed. Further, if the number consumed is `1`, it is allowable to omit the number consumed.

The value of $\gamma_j$ is declared with the `const` keyword, as follows:
```txt
  const 0.058
```

A full reaction is permitted to look like the following (though this code is not as readable as it could be):
```txt
reaction R1
	consume X1 2
	produce X2
	consume X3
	const 0.48
	produce X5 1
```

### Targets

A target is specified using a standard comparison operation. Specifically, a target is a comparison between the count of a particular species in $\mathfrak{X}$ and a desired value. A target is evaluated as a reachability property.

Up to one target may be specified per model, but this may change as tooling improves. Currently allowed comparison operators are `<`, `>`, `<=`, `>=`, `=` or `==`, and `!=`.

The target property is specified using the `target` keyword on its own line:
```txt
target X3 >= 200
```

## Example Files
The following are *equivalent* example files for the CRN/VASS input format.

### CRN
```txt
species S1 init 100
species S2 init 200
species S3 init 300
target S3 = 340
reaction R1
	consume S1 10
	produce S2 3
	const 0.4
reaction R2
	consume S1 1
	produce S3 1
	const 0.6
```

### Generic VASS
```txt
var S1 init 100
var S2 init 200
var S3 init 300
target S3 = 340
transition R1
	decrease S1 10
	increase S2 3
	const 0.4
transition R2
	decrease S1 1
	increase S3 1
	const 0.6
```

