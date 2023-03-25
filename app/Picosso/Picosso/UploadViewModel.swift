//
//  UploadViewModel.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import Foundation
import PhotosUI
import SwiftUI

class UploadViewModel: ObservableObject {
    
    @Published var imageURL: URL
    
    @Published var title = ""
    @Published var artist = ""
    @Published var shouldPad = true
    @Published var shouldOverwrite = true
    
    @Published var isShowingErrorAlert = false
    
    @Published var errorMessage: String? {
        didSet {
            if oldValue == nil {
                isShowingErrorAlert = true
            }
        }
    }
    
    init(imageURL: URL) {
        self.imageURL = imageURL
    }
    
    func upload() async -> Bool {
        var components = URLComponents(url: Endpoint.upload.url, resolvingAgainstBaseURL: true)!
        components.queryItems = [
            .init(name: "title", value: title),
            .init(name: "artist", value: artist),
            .init(name: "pad", value: "\(shouldPad)"),
            .init(name: "overwrite", value: "\(shouldOverwrite)")
        ]
        
        var request = URLRequest(url: components.url!)
        request.httpMethod = "POST"
        
        let boundary = UUID().uuidString
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        
        do {
            guard imageURL.startAccessingSecurityScopedResource() else { return false }
            
            // Create form data
            let formData = NSMutableData()
            formData.append("--\(boundary)\r\n".data(using: String.Encoding.utf8)!)
            formData.append("Content-Disposition: form-data; name=\"file\"; filename=\"\(imageURL.lastPathComponent)\"\r\n".data(using: String.Encoding.utf8)!)
            
            let mimeType = UTType(filenameExtension: imageURL.pathExtension)!.preferredMIMEType!
            formData.append("Content-Type: \(mimeType)\r\n\r\n".data(using: String.Encoding.utf8)!)
            
            formData.append(try Data(contentsOf: imageURL))
            formData.append("\r\n".data(using: String.Encoding.utf8)!)
            formData.append("--\(boundary)--\r\n".data(using: String.Encoding.utf8)!)
            
            // Send upload request
            let (data, response) = try await URLSession.shared.upload(for: request, from: formData as Data)
            imageURL.stopAccessingSecurityScopedResource()
            
            return await MainActor.run {
                if let response = response as? HTTPURLResponse, response.statusCode != 200,
                   let message = String(data: data, encoding: .utf8) {
                    self.errorMessage = message
                    return false
                }
                
                return true
            }
        } catch let error {
            return await MainActor.run {
                imageURL.stopAccessingSecurityScopedResource()
                self.errorMessage = error.localizedDescription
                return false
            }
        }
    }
}
