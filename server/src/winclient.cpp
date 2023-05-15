#include "winclient.h"

#include <QQmlInfo>
#include <QTcpSocket>
#include <bit>

#define WARN_WITH_MSG(cond, msg) \
    if (!(cond)) { \
        qmlWarning(giveMeErr) << msg; \
        return std::nullopt; \
    }

namespace {

namespace in_package_types {
constexpr std::uint8_t Present = 0;
}

namespace out_package_types {
constexpr std::uint8_t Init = 0;
constexpr std::uint8_t Event = 1;
} // namespace out_package_types

namespace event_types {
constexpr std::uint8_t Close = 0;
constexpr std::uint8_t Resize = 1;
} // namespace event_types

namespace transmute {

constexpr void to_be(char *p, std::size_t s)
{
    if constexpr (std::endian::native == std::endian::little) {
        for (std::size_t i = 0; i < s / 2; ++i) {
            const auto tmp = p[i];
            p[i] = p[s - i - 1];
            p[s - i - 1] = tmp;
        }
    }
}

template<typename T>
std::optional<T> read(QIODevice *dev)
{
    if (dev->bytesAvailable() >= sizeof(T)) {
        auto bytes = dev->read(sizeof(T));
        assert(bytes.size() == sizeof(T));
        to_be(bytes.data(), bytes.size());
        const auto ptr = reinterpret_cast<const T *>(bytes.constData());
        return *ptr;
    } else {
        return std::nullopt;
    }
}

template<typename T>
std::size_t write(QIODevice *dev, const T &v)
{
    auto vCopy = v;
    auto ptr = reinterpret_cast<char *>(&vCopy);
    to_be(ptr, sizeof(T));
    const auto bytes = dev->write(ptr, sizeof(T));
    assert(bytes == sizeof(T));
    return bytes;
}

} // namespace transmute

class Package
{
public:
    template<typename T>
    std::size_t write(const T &v)
    {
        return transmute::write(m_dev, v);
    }

    friend void writePackage(QIODevice *dev, std::function<void(Package)> writeFn);

private:
    Package(QIODevice *dev)
        : m_dev(dev)
    {}

    QIODevice *m_dev = nullptr;
};

void writePackage(QIODevice *dev, std::function<void(Package)> writeFn)
{
    QByteArray arr;
    {
        QBuffer buf(&arr);
        buf.open(QIODevice::WriteOnly);
        writeFn(Package(&buf));
    }
    transmute::write(dev, std::uint32_t(arr.size()));
    dev->write(arr);
}

std::optional<QImage::Format> parseFormat(std::uint8_t fmt)
{
    switch (fmt) {
    case 0:
        return QImage::Format_Grayscale8;
    case 1:
        return QImage::Format_ARGB32;
    default:
        return std::nullopt;
    }
}

} // namespace

std::optional<WinClient::Frame> WinClient::parseFrame(QByteArray &&arr,
                                                      std::uint8_t cid,
                                                      QObject *giveMeErr)
{
    QBuffer buf(&arr);
    buf.open(QIODevice::ReadOnly);

    const auto protoVersion = transmute::read<std::uint8_t>(&buf);
    assert(protoVersion);
    const auto clientId = transmute::read<std::uint8_t>(&buf);
    assert(clientId);
    WARN_WITH_MSG(*clientId == cid, "Client id does not match");
    const auto packageType = transmute::read<std::uint8_t>(&buf);
    assert(packageType);
    WARN_WITH_MSG(*packageType == in_package_types::Present,
                  "Server suports only package type " << in_package_types::Present << " (present)");
    const auto format = transmute::read<std::uint8_t>(&buf);
    assert(format);
    const auto pixelSize = transmute::read<std::uint8_t>(&buf);
    assert(pixelSize);
    const auto w = transmute::read<std::uint16_t>(&buf);
    assert(w);
    const auto h = transmute::read<std::uint16_t>(&buf);
    assert(h);
    QByteArray pixels = buf.readAll();

    WARN_WITH_MSG(pixels.size() == *w * *h * *pixelSize,
                  "Parsing request failed: invalid pix array size");
    WARN_WITH_MSG(*protoVersion == 0, "Server supports only protocol version 0");

    const auto fmt = parseFormat(*format);
    WARN_WITH_MSG(fmt, "Unknown pix format: " << *format);
    const QImage image(reinterpret_cast<const uchar *>(pixels.constData()), *w, *h, *fmt);

    WARN_WITH_MSG(!image.isNull(), "Invalid image");
    WARN_WITH_MSG(image.depth() == (*pixelSize * 8),
                  "Image depth not match with received pixel size");

    return Frame{.protoVersion = *protoVersion,
                 .format = *protoVersion,
                 .pixelSize = *pixelSize,
                 .w = *w,
                 .h = *h,
                 .image = image.copy()};
}

WinClient::WinClient(QTcpSocket *socket, uint8_t clientId, QObject *parent)
    : QObject(parent)
    , m_socket(socket)
    , m_id(clientId)
{
    QMetaObject::invokeMethod(
        this,
        [this]() {
            transmute::write(m_socket,
                             std::uint32_t(sizeof(std::uint8_t)   // package type
                                           + sizeof(std::uint8_t) // client id
                                           ));
            transmute::write(m_socket, std::uint8_t(0));
            transmute::write(m_socket, m_id);
        },
        Qt::QueuedConnection);

    connect(m_socket, &QTcpSocket::readyRead, this, [this]() {
        if (!m_currentPackageSize) {
            if (const auto packageSize = transmute::read<std::uint32_t>(m_socket)) {
                m_currentPackageSize = *packageSize;
            }
        }

        if (m_currentPackageSize) {
            if (m_socket->bytesAvailable() >= *m_currentPackageSize) {
                if (const auto frame = parseFrame(m_socket->read(*m_currentPackageSize),
                                                  m_id,
                                                  this)) {
                    m_frame = *frame;
                    if (width() != m_prevW) {
                        emit widthChanged();
                        m_prevW = width();
                    }
                    if (height() != m_prevH) {
                        emit heightChanged();
                        m_prevH = height();
                    }
                    emit imageChanged();
                    emit titleChanged();
                }
                m_currentPackageSize = std::nullopt;
            }
        }
    });

    connect(m_socket, &QTcpSocket::disconnected, this, [this]() {
        m_socket->deleteLater();
        m_socket = nullptr;
        emit dead(QPrivateSignal());
    });
}

void WinClient::sendCloseEvent()
{
    writePackage(m_socket, [](Package p) {
        p.write(out_package_types::Event);
        p.write(event_types::Close);
    });

    //qmlWarning(this) << "TODO: WinClient::sendCloseEvent";
}

QString WinClient::title() const
{
    return "Client " + QString::number(m_id);
}
