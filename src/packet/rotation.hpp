#ifndef SLITHER_SERVER_ROTATION_HPP
#define SLITHER_SERVER_ROTATION_HPP

#include "base.hpp"

struct packet_rotation : public packet_base {
    packet_rotation() = default;
    packet_rotation(packet_t t) : packet_base(t) {}

    uint16_t snakeId = 0; // 3-4, int16, Snake id
    float ang = 0.0f; // 5, int8, ang * pi2 / 256 (current snake angle in radians, clockwise from (1, 0))
    float wang = 0.0f; // 6, int8, wang * pi2 / 256 (target rotation angle snake is heading to)
    float snakeSpeed = 0.0f; // 7, int8, sp / 18 (snake speed?)

    size_t get_size() { return 8; }

    packet_t get_rot_type() const {
        if (wang == 0.0f) {
            if (snakeSpeed == 0.0f) {
                return packet_t_rot_ccw_ang;
            } else if (ang == 0.0f) {
                return packet_t_rot_ccw_sp;
            } else {
                return packet_t_rot_ccw_ang_sp;
            }
        } else {
            if (ang == 0.0f) {
                // TODO: could use last snake direction???
                if (snakeSpeed == 0.0f) {
                    return packet_t_rot_ccw_wang;
                } else {
                    return packet_t_rot_ccw_wang_sp;
                }
            } else {
                if (snakeSpeed == 0.0f) {
                    if (ang >= wang) {
                        return packet_t_rot_ccw_ang_wang;
                    } else {
                        return packet_t_rot_cw_ang_wang;
                    }
                } else {
                    if (ang >= wang) {
                        return packet_t_rot_ccw_ang_wang_sp;
                    } else {
                        return packet_t_rot_cw_ang_wang_sp;
                    }
                }
            }
        }
    }
};

std::ostream& operator<<(std::ostream & out, const packet_rotation & p) {
    out << packet_base(p.get_rot_type(), p.client_time);
    out << write_uint16(p.snakeId);

    if (p.ang != 0.0f) {
        out << write_ang8(p.wang);
    }

    if (p.wang != 0.0f) {
        out << write_ang8(p.wang);
    }

    if (p.snakeSpeed != 0.0f) {
        out << write_uint8(p.snakeSpeed * 18);
    }

    return out;
}

#endif //SLITHER_SERVER_ROTATION_HPP
