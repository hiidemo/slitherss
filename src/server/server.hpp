#ifndef SLITHER_SERVER_SERVER_HPP
#define SLITHER_SERVER_SERVER_HPP

#include "config.hpp"

#include <websocketpp/server.hpp>

typedef websocketpp::connection_hdl connection_hdl;

class server : public websocketpp::server<slither_server_config> {

public:

    template <typename T>
    void send(connection_hdl hdl, T packet,
              websocketpp::frame::opcode::value op,
              websocketpp::lib::error_code & ec) {

        connection_ptr con = get_con_from_hdl(hdl, ec);
        if (ec) { return; }

        std::size_t len = packet.get_size();
        boost::asio::streambuf buf(len);
        buf.prepare(len);

        std::ostream out(&buf);
        out << packet;

        ec = con->send(boost::asio::buffer_cast<void const *>(buf.data()), len, op);
    }

    template <typename T>
    void send_binary(connection_hdl hdl, T packet,
              websocketpp::lib::error_code & ec) {

        send(hdl, packet, websocketpp::frame::opcode::binary, ec);
    }

    template <typename T>
    void send_binary(connection_hdl hdl, T packet) {
        websocketpp::lib::error_code ec;
        send_binary(hdl, packet, ec);
        if (ec) {
            get_alog().write(websocketpp::log::alevel::app, "Write Error: " + ec.message());
        }
    }
};

typedef server::message_ptr message_ptr;

#endif //SLITHER_SERVER_SERVER_HPP
