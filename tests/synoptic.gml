graph [
node
[
  id 0
  label "a"
graphics [
  type "oval"
]
]
node
[
  id 1
  label "a"
graphics [
  type "oval"
]
]
node
[
  id 2
  label "b"
graphics [
  type "oval"
]
]
node
[
  id 3
  label "c"
graphics [
  type "oval"
]
]
node
[
  id 4
  label "d"
graphics [
  type "oval"
]
]
node
[
  id 5
  label "TERMINAL"
graphics [
  type "rhombus"
]
]
node
[
  id 6
  label "INITIAL"
graphics [
  type "rectangle"
]
]
edge
[
  source 0
  target 2
  label "P: 1.00"
,]
edge
[
  source 1
  target 2
  label "P: 0.67"
,]
edge
[
  source 1
  target 4
  label "P: 0.33"
,]
edge
[
  source 2
  target 3
  label "P: 1.00"
,]
edge
[
  source 3
  target 1
  label "P: 0.50"
,]
edge
[
  source 3
  target 3
  label "P: 0.50"
,]
edge
[
  source 4
  target 5
  label "P: 1.00"
,]
edge
[
  source 6
  target 0
  label "P: 1.00"
,]
]
