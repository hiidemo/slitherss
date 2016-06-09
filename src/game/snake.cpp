#include "snake.hpp"

bool snake::tick(long dt) {
    uint8_t changes = 0;

    if (update & (change_dying | change_dead)) {
        return false;
    }

    // rotation
    if (angle != wangle) {
        m_rot_ticks += dt;
        if (m_rot_ticks >= rot_step_interval) {
            const long frames = m_rot_ticks / rot_step_interval;
            const long frames_ticks = frames * rot_step_interval;
            const float rotation = snake_angular_speed * frames_ticks / 1000.0f;
            float dAngle = normalize_angle(wangle - angle);

            if (dAngle > f_pi) {
                dAngle -= f_2pi;
            }

            if (fabs(dAngle) < rotation) {
                angle = wangle;
            } else {
                angle += rotation * (dAngle > 0 ? 1.0f : -1.0f);
            }

            angle = normalize_angle(angle);

            changes |= change_angle;
            m_rot_ticks -= frames_ticks;
        }
    }

    // movement
    m_mov_ticks += dt;
    const long mov_frame_interval = 1000 * move_step_distance / speed;
    if (m_mov_ticks >= mov_frame_interval) {
        const long frames = m_mov_ticks / mov_frame_interval;
        const long frames_ticks = frames * mov_frame_interval;
        const float move_dist = speed * frames_ticks / 1000.0f;
        const size_t len = parts.size();

        // move head
        body& head = parts[0];
        body prev = head;
        // todo: check angles
        head.x += cos(angle) * move_dist;
        head.y += sin(angle) * move_dist;
        for (size_t i = 1; i < len && i < parts_skip_count; ++ i) {
            const body old = parts[i];
            parts[i] = prev;
            prev = old;
        }

        // move intermediate
        for (size_t i = parts_skip_count, j = 0; i < len && i < parts_skip_count + parts_start_move_count; ++ i) {
            body &pt = parts[i];
            const body last = parts[i - 1];
            const body old = pt;

            pt.from(prev);
            const float move_coeff = snake_tail_k * (++ j) / parts_start_move_count;
            pt.offset(move_coeff * (last.x - pt.x), move_coeff * (last.y - pt.y));

            prev = old;
        }

        // move tail
        for (size_t i = parts_skip_count + parts_start_move_count; i < len; ++ i) {
            body &pt = parts[i];
            const body last = parts[i - 1];
            const body old = pt;

            pt.from(prev);
            pt.offset(snake_tail_k * (last.x - pt.x), snake_tail_k * (last.y - pt.y));

            prev = old;
        }

        changes |= change_pos;

        // update speed
        const uint16_t wantedSpeed = acceleration ? boost_speed : base_move_speed;
        if (speed != wantedSpeed) {
            const float sgn = wantedSpeed > speed ? 1.0f : -1.0f;
            const uint16_t acc = static_cast<uint16_t>(speed_acceleration * frames_ticks / 1000.0f);
            if (abs(wantedSpeed - speed) <= acc) {
                speed = wantedSpeed;
            } else {
                speed += sgn * acc;
            }
            changes |= change_speed;
        }

        m_mov_ticks -= frames_ticks;
    }

    if (changes > 0 && changes != update) {
        update |= changes;
        return true;
    }

    return false;
}

std::shared_ptr<snake> snake::getptr() {
    return shared_from_this();
}
