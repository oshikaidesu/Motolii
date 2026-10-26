// osc_in.h: values sent to the machine as Open Sound Control (OSC: TouchDesigner's, Resolume's, a phone's or a
// sensor's way of sending numbers to another program), for a behaviour of the kind osc. Read only through doc.h.
// Listened for as tinyosc's own sample listens (tinyosc/, Martin Roth's, ISC: main.c, a socket on UDP port 9000 not
// blocking, select, recvfrom, a bundle's messages or a lone message), on a thread of its own from the first time a
// behaviour asks; the last number each address was sent kept, and given as it is.
#include "tinyosc/tinyosc.c"
#include <arpa/inet.h>
#include <sys/select.h>
#include <fcntl.h>
#include <unistd.h>
#include <mutex>
#include <thread>
#include <unordered_map>

struct OscIn {
    std::mutex lock;
    std::unordered_map<std::string, double> last;   // (each address's last number)
};
static OscIn& osc_in() {
    static OscIn* in = [] {
        OscIn* o = new OscIn;
        std::thread([o] {
            const int fd = socket(AF_INET, SOCK_DGRAM, 0);
            fcntl(fd, F_SETFL, O_NONBLOCK);
            sockaddr_in sin{};
            sin.sin_family = AF_INET, sin.sin_port = htons(9000), sin.sin_addr.s_addr = INADDR_ANY;
            if (bind(fd, (sockaddr*)&sin, sizeof sin) != 0) return (void)std::fprintf(stderr, "OSC: port 9000 is taken: nothing is heard\n"), (void)close(fd);
            char buffer[2048];
            const auto heard = [o](tosc_message& m) {   // (its first number: a float, a double or an int, the rest left)
                const char kind = tosc_getFormat(&m)[0];
                const double v = kind == 'f' ? tosc_getNextFloat(&m) : kind == 'd' ? tosc_getNextDouble(&m) : kind == 'i' ? tosc_getNextInt32(&m) : NAN;
                if (std::isfinite(v)) { std::lock_guard<std::mutex> hold(o->lock); o->last[tosc_getAddress(&m)] = v; }
            };
            for (;;) {
                fd_set readSet;
                FD_ZERO(&readSet), FD_SET(fd, &readSet);
                timeval timeout{1, 0};
                if (select(fd + 1, &readSet, nullptr, nullptr, &timeout) <= 0) continue;
                sockaddr sa;
                socklen_t sa_len = sizeof(sockaddr_in);
                int len = 0;
                while ((len = (int)recvfrom(fd, buffer, sizeof buffer, 0, &sa, &sa_len)) > 0) {
                    tosc_message m;
                    if (tosc_isBundle(buffer)) {
                        tosc_bundle b;
                        tosc_parseBundle(&b, buffer, len);
                        while (tosc_getNextMessage(&b, &m)) heard(m);
                    } else if (tosc_parseMessage(&m, buffer, len) == 0) heard(m);
                }
            }
        }).detach();
        return o;
    }();
    return *in;
}
// The last number sent to an address ("/track/3/x"), 0 before any.
static double osc_value(const std::string& address) {
    OscIn& o = osc_in();
    std::lock_guard<std::mutex> hold(o.lock);
    const auto at = o.last.find(address);
    return at == o.last.end() ? 0.0 : at->second;
}
