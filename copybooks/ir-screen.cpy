      *> ir-screen.cpy - Screen table
      *> Maps Rust: Screen { name, root: Node }

       01  WS-SCREEN-TABLE.
           05  WS-SCREEN-COUNT       PIC 9(2) VALUE 0.
           05  WS-SCREEN-ENTRY OCCURS 20 TIMES.
               10  SCREEN-NAME       PIC X(30).
               10  SCREEN-ROOT-NODE-IDX
                                     PIC 9(4) VALUE 0.
