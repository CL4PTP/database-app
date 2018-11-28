- simple enough algorithm for candidate keys:

```
function find_candidate_keys(A, F)
    /* A is the set of all attributes and F is the set of functional dependencies */
    K[0] := minimize(A);
    n := 1; /* Number of Keys known so far */
    i := 0; /* Currently processed key */
    while i < n do
        foreach α → β ∈ F do
            /* Build a new potential key from the previous known key and the current FD */
            S := α ∪ (K[i] − β);
            /* Search whether the new potential key is part of the already known keys */ 
            found := false;
            for j := 0 to n-1 do
                if K[j] ⊆ S then found := true;
            /* If not, add if 
            if not found then
                K[n] := minimize(S);
                n := n + 1;
        i := i + 1
    return K
```