%%# Each record chooses what to write: only who owes, and at most 5 of them.
%%# Write it to a .txt. The final count is of the records written.
%%header
Who owes, in %{#name}
%%record
%%if balance < 0
%{name} (%{city}): %{balance}
%%end
%%stop if # >= 5
%%footer
%{count} of the first 5, owing %{sum:balance}
