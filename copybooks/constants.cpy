      *> constants.cpy - COBALT engine limits and type codes
      *> All numeric codes and maximum table sizes

       01  WS-CONSTANTS.
      *> ---- Table size limits ----
           05  WS-MAX-NODES          PIC 9(4) VALUE 500.
           05  WS-MAX-STATE-VARS     PIC 9(4) VALUE 200.
           05  WS-MAX-HANDLERS       PIC 9(4) VALUE 100.
           05  WS-MAX-PARAGRAPHS     PIC 9(4) VALUE 100.
           05  WS-MAX-STMTS-PER-PARA PIC 9(4) VALUE 50.
           05  WS-MAX-SCREENS        PIC 9(4) VALUE 20.
           05  WS-MAX-STATEMENTS     PIC 9(6) VALUE 5000.
           05  WS-MAX-EXPRESSIONS    PIC 9(4) VALUE 2000.
           05  WS-MAX-ARITH-NODES    PIC 9(4) VALUE 1000.
           05  WS-MAX-CONDITIONS     PIC 9(4) VALUE 500.
           05  WS-MAX-WHEN-CLAUSES   PIC 9(4) VALUE 500.
           05  WS-MAX-FOCUS-ELEMS    PIC 9(4) VALUE 100.
           05  WS-MAX-CONDS-PER-FIELD
                                     PIC 9(2) VALUE 10.
           05  WS-MAX-DISPLAY-VALS   PIC 9(2) VALUE 20.
           05  WS-MAX-CONCAT-PAIRS   PIC 9(2) VALUE 10.

      *> ---- Node type codes ----
           05  WS-NODE-TYPES.
               10  WS-NODE-TYPE-CONTAINER
                                     PIC 9(1) VALUE 1.
               10  WS-NODE-TYPE-TEXT PIC 9(1) VALUE 2.
               10  WS-NODE-TYPE-NUMERIC
                                     PIC 9(1) VALUE 3.
               10  WS-NODE-TYPE-BUTTON
                                     PIC 9(1) VALUE 4.

      *> ---- PIC kind codes ----
           05  WS-PIC-KINDS.
               10  WS-PIC-KIND-ALPHA PIC 9(1) VALUE 1.
               10  WS-PIC-KIND-NUMERIC
                                     PIC 9(1) VALUE 2.
               10  WS-PIC-KIND-ALPHABETIC
                                     PIC 9(1) VALUE 3.

      *> ---- Statement type codes ----
           05  WS-STMT-TYPES.
               10  WS-STMT-MOVE      PIC 9(2) VALUE 1.
               10  WS-STMT-ADD       PIC 9(2) VALUE 2.
               10  WS-STMT-SUBTRACT  PIC 9(2) VALUE 3.
               10  WS-STMT-MULTIPLY  PIC 9(2) VALUE 4.
               10  WS-STMT-DIVIDE    PIC 9(2) VALUE 5.
               10  WS-STMT-DISPLAY   PIC 9(2) VALUE 6.
               10  WS-STMT-IF        PIC 9(2) VALUE 7.
               10  WS-STMT-PERFORM   PIC 9(2) VALUE 8.
               10  WS-STMT-STRING-CONCAT
                                     PIC 9(2) VALUE 9.
               10  WS-STMT-EVALUATE  PIC 9(2) VALUE 10.
               10  WS-STMT-PERFORM-UNTIL
                                     PIC 9(2) VALUE 11.
               10  WS-STMT-COMPUTE   PIC 9(2) VALUE 12.
               10  WS-STMT-ACCEPT    PIC 9(2) VALUE 13.
               10  WS-STMT-SET       PIC 9(2) VALUE 14.
               10  WS-STMT-STOP-RUN  PIC 9(2) VALUE 15.

      *> ---- Expression type codes ----
           05  WS-EXPR-TYPES.
               10  WS-EXPR-LITERAL   PIC 9(1) VALUE 1.
               10  WS-EXPR-NUMERIC-LIT
                                     PIC 9(1) VALUE 2.
               10  WS-EXPR-VARIABLE  PIC 9(1) VALUE 3.

      *> ---- Comparison operator codes ----
           05  WS-COMPARE-OPS.
               10  WS-CMP-EQUAL      PIC 9(1) VALUE 1.
               10  WS-CMP-NOT-EQUAL  PIC 9(1) VALUE 2.
               10  WS-CMP-GREATER    PIC 9(1) VALUE 3.
               10  WS-CMP-LESS       PIC 9(1) VALUE 4.
               10  WS-CMP-GREATER-EQ PIC 9(1) VALUE 5.
               10  WS-CMP-LESS-EQ    PIC 9(1) VALUE 6.

      *> ---- Condition type codes ----
           05  WS-COND-TYPES.
               10  WS-COND-COMPARE   PIC 9(1) VALUE 1.
               10  WS-COND-NAME      PIC 9(1) VALUE 2.

      *> ---- Arithmetic operator codes ----
           05  WS-ARITH-OPS.
               10  WS-ARITH-ADD      PIC 9(1) VALUE 1.
               10  WS-ARITH-SUBTRACT PIC 9(1) VALUE 2.
               10  WS-ARITH-MULTIPLY PIC 9(1) VALUE 3.
               10  WS-ARITH-DIVIDE   PIC 9(1) VALUE 4.

      *> ---- ArithExpr node type codes ----
           05  WS-ARITH-NODE-TYPES.
               10  WS-ARITH-NUM      PIC 9(1) VALUE 1.
               10  WS-ARITH-VAR      PIC 9(1) VALUE 2.
               10  WS-ARITH-BINOP    PIC 9(1) VALUE 3.

      *> ---- Event type codes ----
           05  WS-EVENT-TYPES.
               10  WS-EVT-CLICK      PIC 9(1) VALUE 1.
               10  WS-EVT-INPUT      PIC 9(1) VALUE 2.
               10  WS-EVT-NAVIGATE   PIC 9(1) VALUE 3.
               10  WS-EVT-QUIT       PIC 9(1) VALUE 4.
               10  WS-EVT-RESIZE     PIC 9(1) VALUE 5.

      *> ---- Focus kind codes ----
           05  WS-FOCUS-KINDS.
               10  WS-FOCUS-TEXT-INPUT
                                     PIC 9(1) VALUE 1.
               10  WS-FOCUS-NUMERIC-INPUT
                                     PIC 9(1) VALUE 2.
               10  WS-FOCUS-BUTTON   PIC 9(1) VALUE 3.

      *> ---- Accept source codes ----
           05  WS-ACCEPT-SOURCES.
               10  WS-ACCEPT-DATE    PIC 9(1) VALUE 1.
               10  WS-ACCEPT-TIME    PIC 9(1) VALUE 2.
               10  WS-ACCEPT-DAY-OF-WEEK
                                     PIC 9(1) VALUE 3.

      *> ---- Color codes (ANSI terminal, 0-7) ----
           05  WS-COLORS.
               10  WS-COLOR-BLACK    PIC 9(1) VALUE 0.
               10  WS-COLOR-RED      PIC 9(1) VALUE 1.
               10  WS-COLOR-GREEN    PIC 9(1) VALUE 2.
               10  WS-COLOR-YELLOW   PIC 9(1) VALUE 3.
               10  WS-COLOR-BLUE     PIC 9(1) VALUE 4.
               10  WS-COLOR-MAGENTA  PIC 9(1) VALUE 5.
               10  WS-COLOR-CYAN     PIC 9(1) VALUE 6.
               10  WS-COLOR-WHITE    PIC 9(1) VALUE 7.
               10  WS-COLOR-NONE     PIC 9(1) VALUE 9.

      *> ---- Recursion / safety limits ----
           05  WS-MAX-RECURSION-DEPTH
                                     PIC 9(4) VALUE 100.
           05  WS-MAX-LOOP-ITERATIONS
                                     PIC 9(6) VALUE 10000.
