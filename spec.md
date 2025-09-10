# Briefing: Strict Total Order via Safe Pairwise Comparisons (No Contradictions)

Purpose
- Implement an algorithm that builds a strict total order (no ties) from
  pairwise user comparisons.
- Guarantee no contradictions (assuming no misclicks) by only asking
  “safe” comparisons that are not implied by transitivity.
- All items are specified up-front; no insertions later.
- Minimize comparisons to roughly O(n log n) (good for up to ~50 items).

Key idea
- Maintain a current total order of already-placed items.
- Insert each new item by binary search on that order, only comparing the
  new item x against a pivot y from the current list.
- After each answer, shrink the search interval so you never ask an implied
  pair (i.e., transitivity makes many relations unnecessary to ask).

Why no contradictions are possible
- At the time you compare a new item x with any already-placed item y,
  there is no existing path to/from x in the preference graph. Adding
  either edge x > y or y > x cannot create a cycle because x had no edges
  prior to the comparison.
- The algorithm never asks a pair that is already implied by previous
  answers (transitivity is respected by the binary search interval).

Complexity (comparisons)
- Inserting into k items takes about ceil(log2(k + 1)) comparisons.
- Total comparisons ≈ sum over k = 1..(n − 1) of ceil(log2(k + 1))
  ≈ n log2 n − O(n).
- For n = 50, expect low-200s comparisons in typical cases.

Behavioral requirements
- Strict total order only (no ties).
- All items provided at the start.
- Adaptive next question (binary search pivot).
- No “undo”; restart on change of mind/misclick.
- Deterministic result given a fixed initial item order (optionally shuffle
  initial items if you want to reduce order bias).

Interface design
- Provide a core function that sorts an array of strings in place using a
  callback to ask the question “Is a more important than b?”.
- The callback must return 1 if a > b; 0 if b > a. Ties are not allowed.
- The algorithm only calls the callback on safe pairs.

Data structures
- Maintain a temporary dynamic array `order` (array of `char *`) as the
  growing total order.
- Insert each new item `x` by binary searching `order` using the callback.

Edge cases
- Duplicate labels are allowed (items are distinct by index). If you need
  distinct labels, validate before sorting.
- Empty or single-item input returns immediately.
- Input size ≤ 50 recommended for human interaction.

C API specification
- typedef int ask_fn(const char* a, const char* b, void* ctx);
  - Returns 1 if `a` is more important than `b`, else 0.
  - Must never return tie; keep prompting the user until 1 or 0 is decided
    (in an interactive implementation).
- int sort_strict_pairwise(char** items, size_t n, ask_fn* ask, void* ctx);
  - Sorts `items` in place from most to least important.
  - Returns 0 on success; −1 on allocation or callback error.

Reference implementation (C, POSIX)

Library: core algorithm and a default interactive prompt
```c
// file: pairwise_sort.c
// Build a strict total order from pairwise comparisons with no contradictions.
// POSIX libc (getline) is used for convenience.

#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef int (*ask_fn)(const char *a, const char *b, void *ctx);

/*
 * Sorts `items` in place from most (index 0) to least important.
 * - ask(a, b, ctx) must return 1 iff a > b, else 0 (no ties allowed).
 * - Returns 0 on success, -1 on allocation failure or callback error.
 */
int sort_strict_pairwise(char **items, size_t n, ask_fn ask, void *ctx) {
  if (n <= 1) return 0;
  if (!items || !ask) return -1;

  char **order = (char **)malloc(n * sizeof(char *));
  if (!order) return -1;

  size_t k = 0;
  order[k++] = items[0];

  for (size_t i = 1; i < n; ++i) {
    char *x = items[i];
    size_t lo = 0, hi = k;

    while (lo < hi) {
      size_t mid = (lo + hi) / 2;
      char *y = order[mid];
      int a_gt_b = ask(x, y, ctx);

      if (a_gt_b < 0) {
        // Callback signaled an error (optional behavior).
        free(order);
        return -1;
      }

      if (a_gt_b == 1) {
        // x > y -> x must be above y; search upper segment
        hi = mid;
      } else {
        // y > x -> x must be below y; search lower segment
        lo = mid + 1;
      }
    }

    // Insert x at position lo; shift tail up by one.
    if (k - lo > 0) {
      memmove(&order[lo + 1], &order[lo],
              (k - lo) * sizeof(char *));
    }
    order[lo] = x;
    ++k;
  }

  // Copy final order back into items.
  memcpy(items, order, n * sizeof(char *));
  free(order);
  return 0;
}

/* ---------------------- Default interactive "ask" ---------------------- */

typedef struct {
  FILE *in;
  FILE *out;
} prompt_ctx;

/*
 * Prompts the user to decide whether `a` is more important than `b`.
 * Returns 1 if a > b, 0 if b > a. No ties allowed.
 * Returns -1 on I/O error.
 */
int ask_prompt(const char *a, const char *b, void *vctx) {
  prompt_ctx *p = (prompt_ctx *)vctx;
  FILE *in = p && p->in ? p->in : stdin;
  FILE *out = p && p->out ? p->out : stdout;

  char *line = NULL;
  size_t cap = 0;

  for (;;) {
    fprintf(out, "\nCompare the following two items:\n");
    fprintf(out, "  1) %s\n", a);
    fprintf(out, "  2) %s\n", b);
    fprintf(out, "Which is more important? Enter 1 or 2: ");
    fflush(out);

    ssize_t nread = getline(&line, &cap, in);
    if (nread < 0) {
      free(line);
      return -1;
    }

    // Find first non-space character.
    int c = 0;
    for (ssize_t i = 0; i < nread; ++i) {
      if (!isspace((unsigned char)line[i])) {
        c = (unsigned char)line[i];
        break;
      }
    }

    if (c == '1') {
      free(line);
      return 1; // a > b
    }
    if (c == '2') {
      free(line);
      return 0; // b > a
    }

    fprintf(out, "Invalid input. Please enter 1 or 2.\n");
  }
}

/* ---------------------- Example CLI driver (stdin) --------------------- */

static void free_items(char **items, size_t n) {
  if (!items) return;
  for (size_t i = 0; i < n; ++i) free(items[i]);
  free(items);
}

/*
 * Reads non-empty lines from stdin as items, sorts them by interactive
 * comparison, and prints the final order (most to least important).
 *
 * Usage:
 *   - Provide items line-by-line via stdin, end with EOF (Ctrl-D on Unix).
 */
int main(void) {
  char **items = NULL;
  size_t n = 0, cap = 0;

  fprintf(stdout,
          "Enter items to prioritize (one per line). End with EOF.\n");

  char *line = NULL;
  size_t lcap = 0;
  for (;;) {
    ssize_t r = getline(&line, &lcap, stdin);
    if (r < 0) break;

    // Trim trailing newlines and CRs
    while (r > 0 && (line[r - 1] == '\n' || line[r - 1] == '\r')) {
      line[--r] = '\0';
    }
    // Skip empty lines
    if (r == 0) continue;

    if (n == cap) {
      size_t ncap = cap ? cap * 2 : 16;
      char **tmp =
          (char **)realloc(items, ncap * sizeof(char *));
      if (!tmp) {
        perror("realloc");
        free(line);
        free_items(items, n);
        return 1;
      }
      items = tmp;
      cap = ncap;
    }

    char *copy = strdup(line);
    if (!copy) {
      perror("strdup");
      free(line);
      free_items(items, n);
      return 1;
    }
    items[n++] = copy;
  }
  free(line);

  if (n == 0) {
    fprintf(stdout, "No items provided. Exiting.\n");
    return 0;
  }

  prompt_ctx p = {.in = stdin, .out = stdout};
  int rc = sort_strict_pairwise(items, n, ask_prompt, &p);
  if (rc != 0) {
    fprintf(stderr, "Sorting failed.\n");
    free_items(items, n);
    return 1;
  }

  fprintf(stdout, "\nFinal ranking (most to least important):\n");
  for (size_t i = 0; i < n; ++i) {
    fprintf(stdout, "%zu) %s\n", i + 1, items[i]);
  }

  free_items(items, n);
  return 0;
}
```

Build and run (example)
```bash
cc -O2 -Wall -Wextra -o pairwise_sort pairwise_sort.c
./pairwise_sort
```

Notes on safety and implied comparisons
- The algorithm only compares the “currently inserting” item x with pivots
  from the existing ordered list. Before any comparison, x has no edges,
  so either answer is safe and cannot create a cycle.
- After each answer, the binary search interval is adjusted to avoid
  asking any pair that is already implied (e.g., if x > y and y ≥ z, then
  x > z is implied; the interval logic prevents asking about x ? z).

Optional variations
- Use mergesort-style interactive merging of runs; this also ensures only
  safe pairs, with similar comparison counts.
- Randomize initial input order to reduce position bias (keep results
  deterministic by seeding RNG in a fixed manner if needed).
- If you later want “contradiction resolution” (not required here), switch
  to maintaining a DAG and detect cycles on arbitrary pair queries; when
  a cycle is about to be formed, present the minimal cycle to the user to
  resolve. This is not used in the current approach because we never ask
  potentially contradictory pairs.

Testing without human input (simulation)
- You can replace `ask_prompt` with a simulator that compares indices
  against a hidden ground-truth order to verify O(n log n) behavior and
  correctness.

Assumptions recap
- Strict total order (no ties).
- All items provided up-front.
- No misclicks; otherwise restart.
- Up to ~50 items for human interaction is practical.
