// GitHub #414: one entry per element of the approved design (docs/design/approved-goal-ui). Each issue that
// builds an element adds its entry; `props` are computed-style properties that must match the mock exactly,
// width/height/mark within `tol` px, and the element's screenshot crop may differ by at most `maxPixelDiff`.
module.exports=[
  ...require('./typography.cjs'),
  ...require('./evidence_spec.cjs'),
  ...require('./choice_spec.cjs'),
  ...require('./source_review_spec.cjs'),
  ...require('./reference_limit_spec.cjs'),
  ...require('./review_file_spec.cjs'),
  ...require('./frame_spec.cjs'),
  ...require('./workflow_spec.cjs'),
  ...require('./controls_spec.cjs'),
];
