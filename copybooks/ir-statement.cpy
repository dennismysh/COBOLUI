      *> ir-statement.cpy - Global flat statement table
      *> Maps Rust: Statement enum (15 variants)
      *> Nested statements (IF then/else, EVALUATE whens)
      *> use index ranges into this same table.

       01  WS-STATEMENT-TABLE.
           05  WS-STMT-COUNT         PIC 9(6) VALUE 0.
           05  WS-STMT-ENTRY OCCURS 5000 TIMES.
               10  STMT-TYPE-CODE    PIC 9(2).
                   88  STMT-IS-MOVE          VALUE 1.
                   88  STMT-IS-ADD           VALUE 2.
                   88  STMT-IS-SUBTRACT      VALUE 3.
                   88  STMT-IS-MULTIPLY      VALUE 4.
                   88  STMT-IS-DIVIDE        VALUE 5.
                   88  STMT-IS-DISPLAY       VALUE 6.
                   88  STMT-IS-IF            VALUE 7.
                   88  STMT-IS-PERFORM       VALUE 8.
                   88  STMT-IS-STRING-CONCAT VALUE 9.
                   88  STMT-IS-EVALUATE      VALUE 10.
                   88  STMT-IS-PERFORM-UNTIL VALUE 11.
                   88  STMT-IS-COMPUTE       VALUE 12.
                   88  STMT-IS-ACCEPT        VALUE 13.
                   88  STMT-IS-SET           VALUE 14.
                   88  STMT-IS-STOP-RUN      VALUE 15.

      *> --- Operand fields (union-style) ---
      *> MOVE/ADD/SUB/MUL/DIV: source expr + target
               10  STMT-SOURCE-EXPR-IDX
                                     PIC 9(4) VALUE 0.
               10  STMT-TARGET-NAME  PIC X(30).

      *> DISPLAY: list of expression indices
               10  STMT-DISPLAY-COUNT
                                     PIC 9(2) VALUE 0.
               10  STMT-DISPLAY-EXPR-IDX
                                     OCCURS 20 TIMES
                                     PIC 9(4).

      *> IF: condition index, then/else body ranges
               10  STMT-COND-IDX     PIC 9(4) VALUE 0.
               10  STMT-THEN-START   PIC 9(6) VALUE 0.
               10  STMT-THEN-COUNT   PIC 9(4) VALUE 0.
               10  STMT-ELSE-START   PIC 9(6) VALUE 0.
               10  STMT-ELSE-COUNT   PIC 9(4) VALUE 0.

      *> PERFORM: paragraph name
               10  STMT-PARA-NAME    PIC X(30).

      *> STRING-CONCAT: source/delim pairs, into target
               10  STMT-CONCAT-COUNT PIC 9(2) VALUE 0.
               10  STMT-CONCAT-PAIR OCCURS 10 TIMES.
                   15  STMT-CONCAT-SRC-EXPR-IDX
                                     PIC 9(4).
                   15  STMT-CONCAT-DELIM-EXPR-IDX
                                     PIC 9(4).
               10  STMT-CONCAT-INTO  PIC X(30).

      *> EVALUATE: subject expr, when-clause range,
      *>           other-body range
               10  STMT-EVAL-SUBJECT-EXPR-IDX
                                     PIC 9(4) VALUE 0.
               10  STMT-EVAL-WHEN-START
                                     PIC 9(4) VALUE 0.
               10  STMT-EVAL-WHEN-COUNT
                                     PIC 9(4) VALUE 0.
               10  STMT-EVAL-OTHER-START
                                     PIC 9(6) VALUE 0.
               10  STMT-EVAL-OTHER-COUNT
                                     PIC 9(4) VALUE 0.

      *> PERFORM-UNTIL: paragraph name, condition idx
               10  STMT-UNTIL-PARA-NAME
                                     PIC X(30).
               10  STMT-UNTIL-COND-IDX
                                     PIC 9(4) VALUE 0.

      *> COMPUTE: target name, arith-expr root idx
               10  STMT-COMPUTE-TARGET
                                     PIC X(30).
               10  STMT-COMPUTE-ARITH-IDX
                                     PIC 9(4) VALUE 0.

      *> ACCEPT: target name, source code
               10  STMT-ACCEPT-TARGET
                                     PIC X(30).
               10  STMT-ACCEPT-SOURCE
                                     PIC 9(1) VALUE 0.

      *> SET: condition name, value (1=TRUE, 0=FALSE)
               10  STMT-SET-COND-NAME
                                     PIC X(30).
               10  STMT-SET-VALUE    PIC 9(1) VALUE 0.

      *> --- WHEN clause table (for EVALUATE) ---
       01  WS-WHEN-TABLE.
           05  WS-WHEN-COUNT         PIC 9(4) VALUE 0.
           05  WS-WHEN-ENTRY OCCURS 500 TIMES.
               10  WHEN-VALUE-EXPR-IDX
                                     PIC 9(4) VALUE 0.
               10  WHEN-BODY-START   PIC 9(6) VALUE 0.
               10  WHEN-BODY-COUNT   PIC 9(4) VALUE 0.
