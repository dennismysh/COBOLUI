      *> ir-node.cpy - Flattened UI node table
      *> Maps Rust: Node enum (Container, Text, Numeric, Button)
      *> Tree flattened with parent-idx references

       01  WS-NODE-TABLE.
           05  WS-NODE-COUNT         PIC 9(4) VALUE 0.
           05  WS-NODE-ENTRY OCCURS 500 TIMES.
               10  NODE-TYPE-CODE    PIC 9(1).
                   88  NODE-IS-CONTAINER VALUE 1.
                   88  NODE-IS-TEXT      VALUE 2.
                   88  NODE-IS-NUMERIC   VALUE 3.
                   88  NODE-IS-BUTTON    VALUE 4.
               10  NODE-NAME         PIC X(30).
               10  NODE-PARENT-IDX   PIC 9(4) VALUE 0.
               10  NODE-CHILD-COUNT  PIC 9(3) VALUE 0.
      *> PIC clause (Text and Numeric nodes)
               10  NODE-PIC-KIND     PIC 9(1) VALUE 0.
               10  NODE-PIC-WIDTH    PIC 9(3) VALUE 0.
               10  NODE-PIC-DECIMALS PIC 9(2) VALUE 0.
      *> Value and binding
               10  NODE-VALUE        PIC X(80).
               10  NODE-BINDING      PIC X(30).
      *> Button-specific fields
               10  NODE-LABEL        PIC X(40).
               10  NODE-ACTION       PIC X(30).
               10  NODE-NAVIGATE     PIC X(30).
      *> Style
               10  NODE-FG-COLOR     PIC 9(1) VALUE 9.
               10  NODE-BG-COLOR     PIC 9(1) VALUE 9.
