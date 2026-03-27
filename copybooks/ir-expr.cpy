      *> ir-expr.cpy - Expression, Condition, ArithExpr tables
      *> Maps Rust: Expr, Condition, ArithExpr

      *> --- Expression table ---
       01  WS-EXPR-TABLE.
           05  WS-EXPR-COUNT         PIC 9(4) VALUE 0.
           05  WS-EXPR-ENTRY OCCURS 2000 TIMES.
               10  EXPR-TYPE-CODE    PIC 9(1).
                   88  EXPR-IS-LITERAL       VALUE 1.
                   88  EXPR-IS-NUMERIC-LIT   VALUE 2.
                   88  EXPR-IS-VARIABLE      VALUE 3.
               10  EXPR-STR-VALUE    PIC X(80).
               10  EXPR-NUM-VALUE    PIC S9(13)V9(4)
                                     VALUE 0.

      *> --- Condition table ---
       01  WS-CONDITION-TABLE.
           05  WS-COND-COUNT         PIC 9(4) VALUE 0.
           05  WS-COND-ENTRY OCCURS 500 TIMES.
               10  COND-TYPE-CODE    PIC 9(1).
                   88  COND-IS-COMPARE       VALUE 1.
                   88  COND-IS-COND-NAME     VALUE 2.
      *> Compare fields
               10  COND-LEFT-EXPR-IDX
                                     PIC 9(4) VALUE 0.
               10  COND-OP-CODE      PIC 9(1) VALUE 0.
               10  COND-RIGHT-EXPR-IDX
                                     PIC 9(4) VALUE 0.
      *> ConditionName field
               10  COND-NAME-VALUE   PIC X(30).

      *> --- Arithmetic expression node table ---
      *> Tree flattened: each node has type and child indices
       01  WS-ARITH-TABLE.
           05  WS-ARITH-COUNT        PIC 9(4) VALUE 0.
           05  WS-ARITH-ENTRY OCCURS 1000 TIMES.
               10  ARITH-NODE-TYPE   PIC 9(1).
                   88  ARITH-IS-NUM      VALUE 1.
                   88  ARITH-IS-VAR      VALUE 2.
                   88  ARITH-IS-BINOP    VALUE 3.
      *> Num leaf
               10  ARITH-NUM-VALUE   PIC S9(13)V9(4)
                                     VALUE 0.
      *> Var leaf
               10  ARITH-VAR-NAME    PIC X(30).
      *> BinOp interior node
               10  ARITH-OP-CODE     PIC 9(1) VALUE 0.
               10  ARITH-LEFT-IDX    PIC 9(4) VALUE 0.
               10  ARITH-RIGHT-IDX   PIC 9(4) VALUE 0.
