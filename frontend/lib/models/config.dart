/// Mirrors MVP + SHOULD fields accepted by Rust [backend/src/config.rs](../../../../backend/src/config.rs).
class AnalysisWeights {
  const AnalysisWeights({
    required this.phash,
    required this.ssim,
    required this.histogram,
  });

  final double phash;
  final double ssim;
  final double histogram;

  Map<String, dynamic> toJson() => {
        'phash': phash,
        'ssim': ssim,
        'histogram': histogram,
      };
}

class AnalysisConfig {
  AnalysisConfig({
    required this.enablePhash,
    required this.enableSsim,
    required this.enableHistogram,
    required this.enableAlphaCrop,
    required this.enableRotations,
    required this.enableFlip,
    required this.threshold,
    required this.weights,
    required this.hashAlgorithm,
    required this.phashMaxDistance,
    required this.ssimThreshold,
    required this.resizeSize,
    required this.histBins,
    required this.histMethod,
    required this.alphaThreshold,
    this.maxDecodeDimensionPx,
  });

  bool enablePhash;
  bool enableSsim;
  bool enableHistogram;
  bool enableAlphaCrop;
  bool enableRotations;
  bool enableFlip;
  double threshold;
  AnalysisWeights weights;
  String hashAlgorithm;
  int phashMaxDistance;
  double ssimThreshold;
  int resizeSize;
  int histBins;
  String histMethod;
  double alphaThreshold;
  int? maxDecodeDimensionPx;

  factory AnalysisConfig.defaults() => AnalysisConfig(
        enablePhash: true,
        enableSsim: true,
        enableHistogram: true,
        enableAlphaCrop: false,
        enableRotations: false,
        enableFlip: false,
        threshold: 0.85,
        weights: const AnalysisWeights(
          phash: 0.35,
          ssim: 0.45,
          histogram: 0.2,
        ),
        hashAlgorithm: 'sha256',
        phashMaxDistance: 10,
        ssimThreshold: 0.9,
        resizeSize: 256,
        histBins: 512,
        histMethod: 'correlation',
        alphaThreshold: 0.05,
        maxDecodeDimensionPx: null,
      );

  AnalysisConfig copy() => AnalysisConfig(
        enablePhash: enablePhash,
        enableSsim: enableSsim,
        enableHistogram: enableHistogram,
        enableAlphaCrop: enableAlphaCrop,
        enableRotations: enableRotations,
        enableFlip: enableFlip,
        threshold: threshold,
        weights: AnalysisWeights(
          phash: weights.phash,
          ssim: weights.ssim,
          histogram: weights.histogram,
        ),
        hashAlgorithm: hashAlgorithm,
        phashMaxDistance: phashMaxDistance,
        ssimThreshold: ssimThreshold,
        resizeSize: resizeSize,
        histBins: histBins,
        histMethod: histMethod,
        alphaThreshold: alphaThreshold,
        maxDecodeDimensionPx: maxDecodeDimensionPx,
      );

  Map<String, dynamic> toJson() {
    final m = <String, dynamic>{
      'enable_phash': enablePhash,
      'enable_ssim': enableSsim,
      'enable_histogram': enableHistogram,
      'enable_alpha_crop': enableAlphaCrop,
      'enable_rotations': enableRotations,
      'enable_flip': enableFlip,
      'threshold': threshold,
      'weights': weights.toJson(),
      'hash_algorithm': hashAlgorithm,
      'phash_max_distance': phashMaxDistance,
      'ssim_threshold': ssimThreshold,
      'resize_size': resizeSize,
      'hist_bins': histBins,
      'hist_method': histMethod,
      'alpha_threshold': alphaThreshold,
    };
    final px = maxDecodeDimensionPx;
    if (px != null) {
      m['max_decode_dimension_px'] = px;
    }
    return m;
  }
}
