      *> focus-state.cpy - Focus tracking table
      *> Maps Rust: FocusState, FocusableElement, FocusKind

       01  WS-FOCUS-TABLE.
           05  WS-FOCUS-COUNT        PIC 9(4) VALUE 0.
           05  WS-CURRENT-FOCUS-IDX  PIC 9(4) VALUE 0.
           05  WS-FOCUS-ENTRY OCCURS 100 TIMES.
               10  FOCUS-NODE-IDX    PIC 9(4) VALUE 0.
               10  FOCUS-KIND-CODE   PIC 9(1) VALUE 0.
                   88  FOCUS-IS-TEXT-INPUT
                                     VALUE 1.
                   88  FOCUS-IS-NUMERIC-INPUT
                                     VALUE 2.
                   88  FOCUS-IS-BUTTON
                                     VALUE 3.
               10  FOCUS-ELEM-NAME   PIC X(30).
               10  FOCUS-BINDING     PIC X(30).
               10  FOCUS-ACTION      PIC X(30).
               10  FOCUS-NAVIGATE    PIC X(30).
