       IDENTIFICATION DIVISION.
       PROGRAM-ID. COBALT-MAIN.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       COPY "constants.cpy".
       COPY "ir-node.cpy".
       COPY "ir-state.cpy".
       COPY "ir-handler.cpy".
       COPY "ir-paragraph.cpy".
       COPY "ir-statement.cpy".
       COPY "ir-screen.cpy".
       COPY "ir-expr.cpy".
       COPY "runtime-state.cpy".
       COPY "event-record.cpy".
       COPY "focus-state.cpy".
       COPY "render-types.cpy".

       PROCEDURE DIVISION.
       MAIN-PARA.
           DISPLAY "COBALT Phase 1 - All copybooks loaded."
           STOP RUN.
