      *> ir-state.cpy - State variable definitions table
      *> Maps Rust: StateField / StateMap

       01  WS-STATE-TABLE.
           05  WS-STATE-COUNT        PIC 9(4) VALUE 0.
           05  WS-STATE-ENTRY OCCURS 200 TIMES.
               10  STATE-NAME        PIC X(30).
               10  STATE-PIC-KIND    PIC 9(1) VALUE 0.
               10  STATE-PIC-WIDTH   PIC 9(3) VALUE 0.
               10  STATE-PIC-DECIMALS
                                     PIC 9(2) VALUE 0.
               10  STATE-DEFAULT-VALUE
                                     PIC X(80).
      *> Level-88 conditions for this field
               10  STATE-NUM-CONDITIONS
                                     PIC 9(2) VALUE 0.
               10  STATE-CONDITION OCCURS 10 TIMES.
                   15  STATE-COND-NAME
                                     PIC X(30).
                   15  STATE-COND-VALUE
                                     PIC X(80).
