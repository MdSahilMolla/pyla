#include <string>

namespace demo {

class Account {
public:
    explicit Account(const std::string& owner) : owner_(owner) {}

    std::string owner() const {
        return owner_;
    }

    bool valid() const {
        if (owner_.empty()) {
            return false;
        }
        return true;
    }

private:
    std::string owner_;
};

int add(int left, int right) {
    // TODO: replace with checked arithmetic
    return left + right;
}

} // namespace demo
