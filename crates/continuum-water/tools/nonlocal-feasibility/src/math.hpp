#pragma once

#include <array>
#include <cmath>
#include <stdexcept>

namespace nextengine::nonlocal {

struct Vec3 {
    double x = 0.0;
    double y = 0.0;
    double z = 0.0;
};

inline Vec3 operator+(Vec3 a, Vec3 b) {
    return {a.x + b.x, a.y + b.y, a.z + b.z};
}

inline Vec3 operator-(Vec3 a, Vec3 b) {
    return {a.x - b.x, a.y - b.y, a.z - b.z};
}

inline Vec3 operator-(Vec3 a) { return {-a.x, -a.y, -a.z}; }

inline Vec3 operator*(Vec3 a, double scalar) {
    return {a.x * scalar, a.y * scalar, a.z * scalar};
}

inline Vec3 operator*(double scalar, Vec3 a) { return a * scalar; }

inline Vec3 operator/(Vec3 a, double scalar) {
    return {a.x / scalar, a.y / scalar, a.z / scalar};
}

inline Vec3& operator+=(Vec3& a, Vec3 b) {
    a = a + b;
    return a;
}

inline double dot(Vec3 a, Vec3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }

inline double norm_squared(Vec3 a) { return dot(a, a); }

inline double norm(Vec3 a) { return std::sqrt(norm_squared(a)); }

struct Mat3 {
    std::array<double, 9> v{};

    double& operator()(int row, int column) { return v[3 * row + column]; }
    double operator()(int row, int column) const { return v[3 * row + column]; }

    static Mat3 identity() {
        Mat3 result;
        result(0, 0) = 1.0;
        result(1, 1) = 1.0;
        result(2, 2) = 1.0;
        return result;
    }
};

inline Mat3 operator+(Mat3 a, const Mat3& b) {
    for (std::size_t i = 0; i < a.v.size(); ++i) {
        a.v[i] += b.v[i];
    }
    return a;
}

inline Mat3 operator-(Mat3 a, const Mat3& b) {
    for (std::size_t i = 0; i < a.v.size(); ++i) {
        a.v[i] -= b.v[i];
    }
    return a;
}

inline Mat3 operator*(Mat3 a, double scalar) {
    for (double& value : a.v) {
        value *= scalar;
    }
    return a;
}

inline Mat3 operator*(double scalar, Mat3 a) { return a * scalar; }

inline Mat3& operator+=(Mat3& a, const Mat3& b) {
    a = a + b;
    return a;
}

inline Vec3 operator*(const Mat3& matrix, Vec3 vector) {
    return {
        matrix(0, 0) * vector.x + matrix(0, 1) * vector.y + matrix(0, 2) * vector.z,
        matrix(1, 0) * vector.x + matrix(1, 1) * vector.y + matrix(1, 2) * vector.z,
        matrix(2, 0) * vector.x + matrix(2, 1) * vector.y + matrix(2, 2) * vector.z,
    };
}

inline Mat3 outer(Vec3 a, Vec3 b) {
    Mat3 result;
    result(0, 0) = a.x * b.x;
    result(0, 1) = a.x * b.y;
    result(0, 2) = a.x * b.z;
    result(1, 0) = a.y * b.x;
    result(1, 1) = a.y * b.y;
    result(1, 2) = a.y * b.z;
    result(2, 0) = a.z * b.x;
    result(2, 1) = a.z * b.y;
    result(2, 2) = a.z * b.z;
    return result;
}

inline double determinant(const Mat3& matrix) {
    return matrix(0, 0) * (matrix(1, 1) * matrix(2, 2) - matrix(1, 2) * matrix(2, 1))
        - matrix(0, 1) * (matrix(1, 0) * matrix(2, 2) - matrix(1, 2) * matrix(2, 0))
        + matrix(0, 2) * (matrix(1, 0) * matrix(2, 1) - matrix(1, 1) * matrix(2, 0));
}

inline Mat3 inverse_without_regularization(const Mat3& matrix) {
    const double det = determinant(matrix);
    if (!std::isfinite(det) || std::abs(det) <= 1.0e-18) {
        throw std::runtime_error("singular local 3x3 system");
    }

    Mat3 result;
    result(0, 0) = matrix(1, 1) * matrix(2, 2) - matrix(1, 2) * matrix(2, 1);
    result(0, 1) = matrix(0, 2) * matrix(2, 1) - matrix(0, 1) * matrix(2, 2);
    result(0, 2) = matrix(0, 1) * matrix(1, 2) - matrix(0, 2) * matrix(1, 1);
    result(1, 0) = matrix(1, 2) * matrix(2, 0) - matrix(1, 0) * matrix(2, 2);
    result(1, 1) = matrix(0, 0) * matrix(2, 2) - matrix(0, 2) * matrix(2, 0);
    result(1, 2) = matrix(0, 2) * matrix(1, 0) - matrix(0, 0) * matrix(1, 2);
    result(2, 0) = matrix(1, 0) * matrix(2, 1) - matrix(1, 1) * matrix(2, 0);
    result(2, 1) = matrix(0, 1) * matrix(2, 0) - matrix(0, 0) * matrix(2, 1);
    result(2, 2) = matrix(0, 0) * matrix(1, 1) - matrix(0, 1) * matrix(1, 0);
    return result * (1.0 / det);
}

inline bool finite(Vec3 value) {
    return std::isfinite(value.x) && std::isfinite(value.y) && std::isfinite(value.z);
}

inline bool finite(const Mat3& matrix) {
    for (double value : matrix.v) {
        if (!std::isfinite(value)) {
            return false;
        }
    }
    return true;
}

} // namespace nextengine::nonlocal
