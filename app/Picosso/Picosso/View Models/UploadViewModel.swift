//
//  UploadViewModel.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import PhotosUI
import SwiftUI

class UploadViewModel: ObservableObject {
    
    @Published var image: UIImage?
    @Published var title = ""
    @Published var artist = ""
    @Published var useDarkBackground = false
    @Published var canOverwrite = false
    @Published var errorInfo = ErrorAlert.Info()
    
    private let imageURL: URL
    private var imageData: Data?
    
    init(imageURL: URL) {
        self.imageURL = imageURL
        
        // Load local image data
        if imageURL.startAccessingSecurityScopedResource() {
            if let imageData = try? Data(contentsOf: imageURL) {
                self.imageData = imageData
                self.image = UIImage(data: imageData)
            }
            
            imageURL.stopAccessingSecurityScopedResource()
        }
    }
    
    func save() async -> Bool {
        guard let imageData else { return false }
        
        var components = URLComponents(url: Endpoint.upload.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "title", value: title),
            .init(name: "artist", value: artist),
            .init(name: "dark", value: "\(useDarkBackground)"),
            .init(name: "overwrite", value: "\(canOverwrite)")
        ]
        
        var request = URLRequest(url: components.url!)
        request.httpMethod = "POST"
        
        let boundary = UUID().uuidString
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        
        do {
            // Create form data
            let formData = NSMutableData()
            formData.append("--\(boundary)\r\n".data(using: String.Encoding.utf8)!)
            formData.append("Content-Disposition: form-data; name=\"file\"; filename=\"\(imageURL.lastPathComponent)\"\r\n".data(using: String.Encoding.utf8)!)
            
            let mimeType = UTType(filenameExtension: imageURL.pathExtension)!.preferredMIMEType!
            formData.append("Content-Type: \(mimeType)\r\n\r\n".data(using: String.Encoding.utf8)!)
            
            formData.append(imageData)
            formData.append("\r\n".data(using: String.Encoding.utf8)!)
            formData.append("--\(boundary)--\r\n".data(using: String.Encoding.utf8)!)
            
            // Send upload request
            let (data, response) = try await URLSession.shared.upload(for: request, from: formData as Data)
            
            return await MainActor.run {
                if let message = Utils.errorMessage(for: response, with: data) {
                    self.errorInfo.message = message
                    return false
                }
                
                return true
            }
        } catch let error {
            return await MainActor.run {
                imageURL.stopAccessingSecurityScopedResource()
                self.errorInfo.message = error.localizedDescription
                return false
            }
        }
    }
}
